use anyhow::{Context, Result, bail};
use deadpool_postgres::{
	Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime, SslMode,
};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use std::{env, fs, net::IpAddr, time::Duration};
use tokio_postgres::NoTls;
use tokio_postgres_rustls::MakeRustlsConnect;

pub fn pool_from_env() -> Result<Pool> {
	let host = required("DATABASE_HOST")?;
	let port = positive("DATABASE_PORT", 5432_u16)?;
	let user = required("DATABASE_USER")?;
	let dbname = required("DATABASE_NAME")?;
	let password = match setting("DATABASE_USER_PASSWORD_FILE")? {
		Some(path) => {
			if setting("DATABASE_USER_PASSWORD")?.is_some() {
				bail!(
					"Set only one of DATABASE_USER_PASSWORD and DATABASE_USER_PASSWORD_FILE"
				);
			}
			let value = fs::read_to_string(path)
				.context("Cannot read DATABASE_USER_PASSWORD_FILE")?;
			let value = value.trim_end_matches(['\r', '\n']);
			if value.trim().is_empty() {
				bail!("DATABASE_USER_PASSWORD_FILE must contain a password");
			}
			value.to_owned()
		}
		None => required("DATABASE_USER_PASSWORD")?,
	};
	let loopback = host.eq_ignore_ascii_case("localhost")
		|| host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback());
	let tls_mode = setting("DATABASE_TLS_MODE")?
		.unwrap_or_else(|| if loopback { "disable" } else { "verify-full" }.into());

	let mut pool = PoolConfig::new(positive("DATABASE_POOL_MAX_SIZE", 16_usize)?);
	pool.timeouts.wait = Some(timeout("DATABASE_POOL_WAIT_TIMEOUT_SECS", 5)?);
	pool.timeouts.create = Some(timeout("DATABASE_POOL_CREATE_TIMEOUT_SECS", 5)?);
	pool.timeouts.recycle = Some(timeout("DATABASE_POOL_RECYCLE_TIMEOUT_SECS", 3)?);
	let mut config = Config {
		host: Some(host),
		port: Some(port),
		user: Some(user),
		dbname: Some(dbname),
		password: Some(password),
		pool: Some(pool),
		manager: Some(ManagerConfig {
			recycling_method: RecyclingMethod::Verified,
		}),
		..Config::default()
	};
	match tls_mode.as_str() {
		"disable" if loopback => {
			config.ssl_mode = Some(SslMode::Disable);
			Ok(config.create_pool(Some(Runtime::Tokio1), NoTls)?)
		}
		"disable" => {
			bail!("DATABASE_TLS_MODE=disable is only allowed for loopback addresses")
		}
		"verify-full" => {
			let pem = fs::read(required("DATABASE_TLS_CA_FILE")?)
				.context("Cannot read DATABASE_TLS_CA_FILE")?;
			let mut roots = rustls::RootCertStore::empty();
			for cert in CertificateDer::pem_slice_iter(&pem) {
				roots
					.add(cert.context("Invalid PEM in DATABASE_TLS_CA_FILE")?)
					.context("Invalid CA certificate in DATABASE_TLS_CA_FILE")?;
			}
			if roots.is_empty() {
				bail!("DATABASE_TLS_CA_FILE must contain a PEM CA certificate");
			}
			let tls = rustls::ClientConfig::builder_with_provider(
				rustls::crypto::aws_lc_rs::default_provider().into(),
			)
			.with_safe_default_protocol_versions()?
			.with_root_certificates(roots)
			.with_no_client_auth();
			config.ssl_mode = Some(SslMode::Require);
			Ok(config
				.create_pool(Some(Runtime::Tokio1), MakeRustlsConnect::new(tls))?)
		}
		_ => bail!("DATABASE_TLS_MODE must be disable or verify-full"),
	}
}

fn setting(name: &str) -> Result<Option<String>> {
	match env::var(name) {
		Ok(value) => Ok(Some(value)),
		Err(env::VarError::NotPresent) => Ok(None),
		Err(_) => bail!("{name} must be valid Unicode"),
	}
}

fn required(name: &str) -> Result<String> {
	setting(name)?
		.filter(|value| !value.trim().is_empty())
		.with_context(|| format!("{name} must be set and non-empty"))
}

fn positive<T>(name: &str, default: T) -> Result<T>
where
	T: std::str::FromStr + Default + PartialOrd,
{
	match setting(name)? {
		None => Ok(default),
		Some(value) => value
			.parse::<T>()
			.ok()
			.filter(|value| *value > T::default())
			.with_context(|| format!("{name} must be a positive integer in range")),
	}
}

fn timeout(name: &str, default: u32) -> Result<Duration> {
	Ok(Duration::from_secs(positive(name, default)?.into()))
}

#[cfg(test)]
pub fn create_database_pool(
	host: String,
	port: String,
	dbname: String,
	user: String,
	password: String,
) -> Pool {
	Config {
		host: Some(host),
		port: Some(port.parse().unwrap()),
		dbname: Some(dbname),
		user: Some(user),
		password: Some(password),
		..Config::default()
	}
	.create_pool(Some(Runtime::Tokio1), NoTls)
	.unwrap()
}
