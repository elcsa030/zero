// TODO: Rename this file to change the name of this method from METHOD_NAME

#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
{% unless risc0_std -%}
#![no_std]  // std support is experimental
{% endunless %}
risc0_zkvm::guest::entry!(main);

pub fn main() {
    // TODO: Implement your guest code here
}
