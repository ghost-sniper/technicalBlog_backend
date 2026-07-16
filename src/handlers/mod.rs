use actix_web::web;
pub mod category;
pub mod tag;
pub fn configure_app(cfg: &mut web::ServiceConfig) {
	cfg.route("/categories", web::get().to(category::get_all_categories))
		.route("/tags", web::get().to(tag::get_all_tags));
}
