pub trait Harness {
    fn test(&self);
}

pub struct AddressLookupTableProgramTestHarness;
impl Harness for AddressLookupTableProgramTestHarness {
    fn test(&self) {
        println!("Testing AddressLookupTableProgram");
    }
}

pub struct ConfigProgramTestHarness;
impl Harness for ConfigProgramTestHarness {
    fn test(&self) {
        println!("Testing ConfigProgram");
    }
}

pub struct FeatureGateProgramHarness;
impl Harness for FeatureGateProgramHarness {
    fn test(&self) {
        println!("Testing FeatureGateProgram");
    }
}
