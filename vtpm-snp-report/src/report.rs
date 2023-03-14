use super::certs::Vcek;
use openssl::{ecdsa::EcdsaSig, sha::Sha384};
use sev::firmware::guest::types::{AttestationReport, Signature};
use std::error::Error;

pub trait Validateable {
    fn validate(&self, vcek: &Vcek) -> Result<(), Box<dyn Error>>;
}

impl Validateable for AttestationReport {
    fn validate(&self, vcek: &Vcek) -> Result<(), Box<dyn Error>> {
        if !is_tcb_data_valid(self) {
            return Err("TCB data is not valid".into());
        }

        let report_sig: EcdsaSig = (&self.signature).try_into()?;
        let vcek_pubkey = vcek.0.public_key()?.ec_key()?;

        let mut hasher = Sha384::new();
        let base_message = get_report_base(self)?;
        hasher.update(&base_message);
        let base_message_digest = hasher.finish();

        if !report_sig.verify(&base_message_digest, &vcek_pubkey)? {
            return Err("Measurement signature is not valid".into());
        }
        Ok(())
    }
}

pub fn parse(bytes: &[u8]) -> Result<AttestationReport, Box<dyn Error>> {
    let decoded: AttestationReport = bincode::deserialize(bytes)?;
    Ok(decoded)
}

fn is_tcb_data_valid(report: &AttestationReport) -> bool {
    report.reported_tcb == report.committed_tcb
}

fn get_report_base(report: &AttestationReport) -> Result<Vec<u8>, Box<dyn Error>> {
    let report_len = std::mem::size_of::<AttestationReport>();
    let signature_len = std::mem::size_of::<Signature>();
    let bytes = bincode::serialize(report)?;
    let report_bytes_without_sig = &bytes[0..(report_len - signature_len)];
    Ok(report_bytes_without_sig.to_vec())
}