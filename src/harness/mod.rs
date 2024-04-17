use crate::validator::ValidatorContext;

pub trait Harness {
    fn test(&self, context: &ValidatorContext);
}

pub struct AddressLookupTableProgramTestHarness;
impl Harness for AddressLookupTableProgramTestHarness {
    fn test(&self, _context: &ValidatorContext) {
        println!("Testing AddressLookupTableProgram");
    }
}

pub struct ConfigProgramTestHarness;
impl Harness for ConfigProgramTestHarness {
    fn test(&self, _context: &ValidatorContext) {
        println!("Testing ConfigProgram");
    }
}

pub struct FeatureGateProgramHarness;
impl Harness for FeatureGateProgramHarness {
    fn test(&self, _context: &ValidatorContext) {
        println!("Testing FeatureGateProgram");
    }
}
