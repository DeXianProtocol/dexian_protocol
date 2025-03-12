#![allow(dead_code)]

use radix_engine::blueprints::package;
use scrypto::prelude::Package;
use scrypto_test::prelude::*;
use super::Resources;
use ::common::*;

use std::path::Path;

fn check_compile(
    package_path: &str,
    package_name: &str,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
) -> (Vec<u8>, PackageDefinition){
    let tests_compiled_dir = Path::new("tests").join("compiled");
    let wasm_path = tests_compiled_dir.join(format!("{}.wasm", package_name));
    let rpd_path = tests_compiled_dir.join(format!("{}.rpd", package_name));

    if wasm_path.exists() && rpd_path.exists() {
        let code = std::fs::read(&wasm_path).expect("failed to read WASM file");
        let definition: PackageDefinition = manifest_decode(
            &std::fs::read(&rpd_path).expect("Failed to read RPD file")
        ).expect("Failed to decode RPD file");
        return (code, definition);
    } else {
        let (code, definition) = Compile::compile_with_env_vars(
            package_path,
            envs.clone(),
            CompileProfile::Standard,
            use_coverage
        );

        let compiled_path = Path::new(package_path).join("target").join("wasm32-unknown-unknown").join("release");
        let wasm_path = compiled_path.join(format!("{}.wasm", package_name));
        let rpd_path = compiled_path.join(format!("{}.rpd", package_name));

        std::fs::create_dir_all(&tests_compiled_dir).expect("Failed to create tests/compiled directory");
        std::fs::copy(&wasm_path, tests_compiled_dir.join(format!("{}.wasm", package_name))).expect("Failed to copy WASM WASM file to tests/compiled");
        std::fs::copy(&rpd_path, tests_compiled_dir.join(format!("{}.rpd", package_name))).expect("failed to copy RPD file tests/compiled");

        return (code, definition);
    }
}

#[derive(Clone)]
pub struct Components {
    pub keeper_package: PackageAddress,
    pub keeper_component: ComponentAddress,
    pub interest_package: PackageAddress,
    pub interest_component: ComponentAddress,
    pub oracle_package: PackageAddress,
    pub oracle_component: ComponentAddress,
    pub protocol_package: PackageAddress,
    pub cdp_component: ComponentAddress,
    pub earning_component: ComponentAddress,

}

pub fn create_components(
    account: ComponentAddress,
    public_key: Secp256k1PublicKey,
    resources: &Resources,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> Components {
    let use_coverage = true;
    let encoder = &AddressBech32Encoder::for_simulator();

    let envs = &mut btreemap!{
        "RUSTFLAGS".to_owned() => "".to_owned(),
        "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
        "OWNER_RESOURCE".to_owned() => resources.owner_resource.to_string(encoder),
        "AUTHORITY_RESOURCE".to_owned() => resources.authority_resource.to_string(encoder),
        "BASE_AUTHORITY_RESOURCE".to_owned() => resources.base_authority_resource.to_string(encoder),
        "BASE_RESOURCE".to_owned() => resources.base_resource.to_string(encoder),
    };

    let pub_key_str = "a5bc3d9296bda1e52f96bf0a65238998877dbddb0703bd37ef1f18a6ffce458a";
    let (keeper_package, keeper_component) = create_keeper(resources, envs, use_coverage, encoder, ledger);
    let (interest_package, interest_component) = create_interest(resources, envs, use_coverage, encoder, ledger);
    let (oracle_package, oracle_component) = create_oracle(pub_key_str, resources, envs, use_coverage, encoder, ledger);
    let (protocol_package, earning_component, cdp_component) = create_protocol(resources, envs, use_coverage, encoder, ledger);
    Components { 
        keeper_package, 
        keeper_component, 
        interest_package,
        interest_component,
        oracle_package, 
        oracle_component,
        protocol_package,
        cdp_component,
        earning_component
     }
}

fn create_keeper(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress){
    let keeper_package = ledger.publish_package(
        check_compile("../keeper", "keeper", envs, use_coverage), 
        BTreeMap::new(),
        resources.owner_role.clone()
    );

    let keeper_component = ledger.call_function(
        keeper_package,
        "ValidatorKeeper",
        "instantiate", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("KEEPER_PACKAGE".to_owned(), keeper_package.to_string(encoder));
    envs.insert("KEEPER_COMPONENT".to_owned(), keeper_component.to_string(encoder));

    (keeper_package, keeper_component)
}

fn create_interest(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress){
    let interest_package = ledger.publish_package(
        check_compile("../interest", "interest", envs, use_coverage), 
        BTreeMap::new(),
        resources.owner_role.clone()
    );

    let interest_component = ledger.call_function(
        interest_package,
        "DefInterestModel",
        "instantiate", 
        manifest_args!(resources.owner_role.clone(), dec!("0.2"), dec!("0.5"), dec!("0.55"), dec!("0.45"))
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("INTEREST_PACKAGE".to_owned(), interest_package.to_string(encoder));
    envs.insert("INTEREST_COMPONENT".to_owned(), interest_component.to_string(encoder));

    (interest_package, interest_component)
}

fn create_oracle(
    pub_key_str: &str,
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress){
    let oracle_package = ledger.publish_package(
        check_compile("../oracle", "oracle", envs, use_coverage), 
        BTreeMap::new(),
        resources.owner_role.clone()
    );

    let oracle_component = ledger.call_function(
        oracle_package,
        "PriceOracle",
        "instantiate", 
        manifest_args!(resources.owner_role.clone(),  pub_key_str, 3000u64)
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("ORACLE_PACKAGE".to_owned(), oracle_package.to_string(encoder));
    envs.insert("ORACLE_COMPONENT".to_owned(), oracle_component.to_string(encoder));

    (oracle_package, oracle_component)
}

fn create_protocol(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress, ComponentAddress){
    let protocol_package = ledger.publish_package(
        check_compile("../protocol", "protocol", envs, use_coverage), 
        BTreeMap::new(),
        resources.owner_role.clone()
    );

    let binding = ledger.call_function(
        protocol_package,
        "StakingEarning",
        "instantiate", 
        manifest_args!(resources.owner_role.clone())
    );
    let component_addresses = binding.expect_commit_success().new_component_addresses();
    
    let staking_pool = component_addresses[0];
    let earning_component = component_addresses[1];

    let cdp_component = ledger.call_function(
        protocol_package, 
        "CollateralDebtManager", 
        "instantiate",
        manifest_args!(
            resources.owner_role.clone(),
            rule!(require(global_caller(earning_component)))       
        )
    ).expect_commit_success().new_component_addresses()[0];
    
    
    envs.insert("PROTOCOL_PACKAGE".to_owned(), protocol_package.to_string(encoder));
    envs.insert("EARNING_COMPONENT".to_owned(), earning_component.to_string(encoder));
    envs.insert("CDP_COMPONENT".to_owned(), cdp_component.to_string(encoder));

    (protocol_package, earning_component, cdp_component)
}