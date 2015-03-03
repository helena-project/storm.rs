/*
 * Cryptography related interfaces.
 */

pub enum SymmetricMode {
    ElectronicCodeBook,
    CipherBlockChaining,
    OutputFeedback,
    CipherFeedback,
    Counter
}

pub trait Symmetric128 {
    fn set_mode (&mut self, SymmetricMode);
    fn set_key (&mut self, key: &[u32; 4]);
    fn set_iv (&mut self, iv: &[u32; 4]);

    /// Plaintext is encrypted up to % 4 of `min(plaintext.len(), ciphertext.len())`.
    fn encrypt_sync (&mut self, plaintext: &[u32], ciphertext: &mut[u32]);

    /// Ciphertext is decrypted up to % 4 of `min(ciphertext.len(), plaintext.len())`.
    fn decrypt_sync (&mut self, ciphertext: &[u32], plaintext: &mut[u32]);
}

pub trait Symmetric192 {
    fn set_mode (&mut self, SymmetricMode);
    fn set_key (&mut self, key: &[u32; 6]);
    fn set_iv (&mut self, iv: &[u32; 6]);

    /// Plaintext is encrypted up to % 6 of `min(plaintext.len(), ciphertext.len())`.
    fn encrypt_sync (&mut self, plaintext: &[u32], ciphertext: &mut[u32]);

    /// Ciphertext is decrypted up to % 6 of `min(ciphertext.len(), plaintext.len())`.
    fn decrypt_sync (&mut self, ciphertext: &[u32], plaintext: &mut[u32]);
}

pub trait Symmetric256 {
    fn set_mode (&mut self, SymmetricMode);
    fn set_key (&mut self, key: &[u32; 8]);
    fn set_iv (&mut self, iv: &[u32; 8]);

    /// Plaintext is encrypted up to % 8 of `min(plaintext.len(), ciphertext.len())`.
    fn encrypt_sync (&mut self, plaintext: &[u32], ciphertext: &mut[u32]);

    /// Ciphertext is decrypted up to % 8 of `min(ciphertext.len(), plaintext.len())`.
    fn decrypt_sync (&mut self, ciphertext: &[u32], plaintext: &mut[u32]);
}

