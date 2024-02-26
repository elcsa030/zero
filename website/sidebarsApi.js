// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
export default {
  api: [
    "introduction",
    {
      type: "category",
      label: "zkVM",
      link: {
        type: `doc`,
        id: "zkvm/zkvm_overview",
      },
      collapsed: false,
      items: [
        {
          type: "doc",
          label: "Quick Start",
          id: "zkvm/quickstart",
        },
        {
          type: "doc",
          label: "Dev Mode",
          id: "zkvm/dev-mode",
        },
        {
          type: "doc",
          label: "Installation",
          id: "zkvm/install",
        },
        {
          type: "doc",
          label: "Rust Resources",
          id: "zkvm/rust-resources",
        },
        {
          type: "doc",
          label: "Guest Code 101",
          id: "zkvm/guest-code-101",
        },
        {
          type: "doc",
          label: "Host Code 101",
          id: "zkvm/host-code-101",
        },
        {
          type: "doc",
          label: "Receipts",
          id: "zkvm/receipts",
        },
        {
          type: "doc",
          label: "Guest Optimization",
          id: "zkvm/optimization",
        },
        {
          type: "doc",
          label: "Cryptography Acceleration",
          id: "zkvm/acceleration",
        },
        {
          type: "doc",
          label: "Profiling",
          id: "zkvm/profiling",
        },
        {
          type: "doc",
          label: "zkVM technical specification",
          id: "zkvm/zkvm-specification",
        },
        {
          type: "doc",
          label: "Performance Benchmarks",
          id: "zkvm/benchmarks",
        },
        {
          type: "category",
          label: "Tutorials",
          link: {
            type: `doc`,
            id: "zkvm/tutorials/overview",
          },
          collapsed: false,
          items: [
            {
              type: "doc",
              label: "Hello World Tutorial",
              id: "zkvm/tutorials/hello-world",
            },
          ],
        },
        {
          type: "doc",
          label: "Examples",
          id: "zkvm/examples",
        },
        {
          type: "link",
          label: "API Reference Docs",
          href: "https://docs.rs/risc0-zkvm/",
        },
        {
          type: "link",
          label: "Source code",
          href: "https://github.com/risc0/risc0",
        },
      ],
    },
    {
      type: "category",
      label: "Bonsai",
      link: {
        type: `doc`,
        id: "bonsai/bonsai-overview",
      },
      collapsed: false,
      items: [
        {
          type: "doc",
          label: "Quick Start",
          id: "bonsai/quickstart",
        },
        {
          type: "doc",
          label: "RISC Zero on Ethereum",
          id: "bonsai/bonsai-on-eth",
        },
        {
          type: "doc",
          label: "A Blockchain Dev's Guide to zkVM Development",
          id: "bonsai/blockchain-zkvm-guide",
        },
        {
          type: "doc",
          label: "Ethereum Examples",
          id: "bonsai/eth-examples",
        },
        {
          type: "category",
          label: "REST API",
          link: {
            type: `doc`,
            id: "bonsai/rest-api",
          },
          collapsed: false,
          items: [
            {
              type: "link",
              label: "API Reference Docs",
              href: "https://api.bonsai.xyz/swagger-ui/",
            },
          ],
        },
        {
          type: "category",
          label: "Verifier Contract",
          link: {
            type: `doc`,
            id: "bonsai/verifier-contract/overview",
          },
          collapsed: false,
          items: [
            {
              type: "doc",
              label: "About",
              id: "bonsai/verifier-contract/about",
            },
            {
              type: "doc",
              label: "Interface",
              id: "bonsai/verifier-contract/interface",
            },
            {
              type: "doc",
              label: "Example Usage",
              id: "bonsai/verifier-contract/example",
            },
            {
              type: "doc",
              label: "Versioning",
              id: "bonsai/verifier-contract/versioning",
            },
            {
              type: "doc",
              label: "Gas costs",
              id: "bonsai/verifier-contract/gas-costs",
            },
            {
              type: "doc",
              label: "Contract Addresses",
              id: "bonsai/verifier-contract/addresses",
            },
          ],
        },
      ],
    },
  ],
};
