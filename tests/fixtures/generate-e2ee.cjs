// Run from any directory: node tests/fixtures/generate-e2ee.cjs
// Requires the frontend dependencies. All keys generated here are test-only.
const { webcrypto, publicEncrypt, constants } = require('node:crypto');
const { writeFileSync } = require('node:fs');
const { join } = require('node:path');

global.window = { crypto: webcrypto, atob, btoa };
const { encryptPlaintext } = require('../../../technicalBlog_Frontend/node_modules/@chatereum/react-e2ee/bin/lib/plaintext/plaintext.lib.js');
const base64 = (bytes) => Buffer.from(bytes).toString('base64');
const pem = (label, bytes) => `-----BEGIN ${label}-----\n${base64(bytes).match(/.{1,64}/g).join('\n')}\n-----END ${label}-----`;

async function main() {
  const fixtures = [];
  for (const bits of [2048, 4096]) {
    const pair = await webcrypto.subtle.generateKey({
      name: 'RSA-OAEP', modulusLength: bits,
      publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256',
    }, true, ['encrypt', 'decrypt']);
    const private_key = pem('PRIVATE KEY', await webcrypto.subtle.exportKey('pkcs8', pair.privateKey));
    const public_key = pem('PUBLIC KEY', await webcrypto.subtle.exportKey('spki', pair.publicKey));
    const plaintext = 'test-password-中文-🔐';
    const payload = await encryptPlaintext({ public_key, plain_text: plaintext });
    const valid = { name: `frontend-${bits}`, private_key, ...payload, plaintext };
    fixtures.push(valid);
    if (bits !== 2048) continue;
    const invalid = (name, changes) => fixtures.push({ ...valid, name, plaintext: null, ...changes });
    invalid('invalid-base64', { aes_key: '%%%invalid%%%' });
    invalid('invalid-pem', { private_key: 'invalid PEM' });
    invalid('wrong-pem-label', { private_key: private_key.replaceAll('PRIVATE KEY', 'PUBLIC KEY') });
    invalid('short-iv', { iv: base64(new Uint8Array(15)) });
    invalid('long-iv', { iv: base64(new Uint8Array(17)) });
    invalid('empty-ciphertext', { cipher_text: '' });
    invalid('unaligned-ciphertext', { cipher_text: base64(new Uint8Array(17)) });
    invalid('invalid-rsa-ciphertext', { aes_key: base64(new Uint8Array(256)) });
    invalid('truncated-rsa-ciphertext', { aes_key: base64(Buffer.from(payload.aes_key, 'base64').subarray(1)) });
    invalid('wrong-oaep-hash', { aes_key: base64(publicEncrypt({
      key: public_key, padding: constants.RSA_PKCS1_OAEP_PADDING, oaepHash: 'sha1',
    }, Buffer.from(base64(new Uint8Array(32))))) });
    for (const length of [16, 24, 31, 33]) {
      const wrapped = await webcrypto.subtle.encrypt({ name: 'RSA-OAEP' }, pair.publicKey, Buffer.from(base64(new Uint8Array(length))));
      invalid(`wrong-aes-key-length-${length}`, { aes_key: base64(wrapped) });
    }
    const notBase64 = await webcrypto.subtle.encrypt({ name: 'RSA-OAEP' }, pair.publicKey, Buffer.from('%%%'));
    invalid('wrapped-key-not-base64', { aes_key: base64(notBase64) });
    // Two blocks with known padding: alter the preceding block to make the
    // final PKCS#7 byte zero, guaranteeing rejection rather than relying on chance.
    const padded = await encryptPlaintext({ public_key, plain_text: 'a'.repeat(16) });
    const brokenPadding = Buffer.from(padded.cipher_text, 'base64');
    brokenPadding[15] ^= 16;
    invalid('invalid-aes-padding', { ...padded, cipher_text: base64(brokenPadding) });
    const aes = await webcrypto.subtle.generateKey({ name: 'AES-CBC', length: 256 }, true, ['encrypt']);
    const rawKey = await webcrypto.subtle.exportKey('raw', aes);
    const wrapped = await webcrypto.subtle.encrypt({ name: 'RSA-OAEP' }, pair.publicKey, Buffer.from(base64(rawKey)));
    const iv = new Uint8Array(16);
    const invalidUtf8 = await webcrypto.subtle.encrypt({ name: 'AES-CBC', iv }, aes, new Uint8Array([0xff]));
    invalid('invalid-utf8', { aes_key: base64(wrapped), iv: base64(iv), cipher_text: base64(invalidUtf8) });
    const empty = await encryptPlaintext({ public_key, plain_text: '' });
    fixtures.push({ name: 'empty-plaintext', private_key, ...empty, plaintext: '' });
  }
  // Reuse a different test key to verify that mismatched private keys fail.
  fixtures.push({ ...fixtures[0], name: 'wrong-private-key', private_key: fixtures.at(-1).private_key, plaintext: null });
  writeFileSync(join(__dirname, 'e2ee.json'), JSON.stringify(fixtures, null, 2) + '\n');
  console.log(`Generated ${fixtures.length} frontend compatibility and rejection cases.`);
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
