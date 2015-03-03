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

pub trait Symmetric {
    /// TODO: (SymmetricMode, block_size) must be supported by underlying chip
    fn set_mode (&mut self, SymmetricMode, block_size: usize);

    /// Key length must equal block_size from `set_mode`
    fn set_key (&mut self, key: &[u32]);

    /// IV Length must equal block_size from `set_mode`
    fn set_iv (&mut self, iv: &[u32]);

    /// `length` must be an integer multiple of block_size from `set_mode`
    fn encrypt_sync (&mut self, plaintext: &[u32], ciphertext: &mut[u32]);

    /// `length` must be an integer multiple of block_size from `set_mode`
    fn decrypt_sync (&mut self, ciphertext: &[u32], plaintext: &mut[u32]);
}
