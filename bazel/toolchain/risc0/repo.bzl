TOOLCHAINS = [
    struct(
        archive = "riscv32im-darwin-arm64",
        host_arch = "@platforms//cpu:arm64",
        host_os = "@platforms//os:osx",
        sha256 = "203255aa7466a2f049169e9de4b3a155c03a0f4549e8c30593d86c7d6b93ed9b",
    ),
    struct(
        archive = "riscv32im-darwin-x86_64",
        host_arch = "@platforms//cpu:x86_64",
        host_os = "@platforms//os:osx",
        sha256 = "6cad684a275c32b4cd2a75643c0417447807245a60c2130422cc14e33eb7e8a8",
    ),
    struct(
        archive = "riscv32im-ubuntu18-amd64",
        host_arch = "@platforms//cpu:x86_64",
        host_os = "@platforms//os:linux",
        sha256 = "bec46711fa6f5c7bd8e09610b0b4643136a6b47318a70fea0817e6fce02233f2",
    ),
]

def _repo_impl(rctx):
    rctx.template(
        "BUILD.bazel",
        Label("//bazel/toolchain/risc0:repo.tpl.BUILD"),
        {
            "{workspace}": rctx.name,
        },
    )

    rctx.file(
        "gcc_toolchain.bzl",
        content = rctx.read(Label("//bazel/toolchain/risc0:gcc_toolchain.bzl")),
        executable = False,
    )

    rctx.download_and_extract(
        url = "https://github.com/risc0/toolchain/releases/download/{version}/{archive}.tar.xz".format(
            archive = rctx.attr.archive,
            version = rctx.attr.version,
        ),
        sha256 = rctx.attr.sha256,
        stripPrefix = rctx.attr.archive,
    )

risc0_toolchain_repo = repository_rule(
    attrs = {
        "version": attr.string(
            mandatory = True,
            doc = "One of the versions of the RISC-V toolchain from https://github.com/risc0/toolchain",
        ),
        "archive": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
    },
    implementation = _repo_impl,
)

def _proxy_impl(rctx):
    rctx.template(
        "BUILD.bazel",
        Label("//bazel/toolchain/risc0:proxy.tpl.BUILD"),
        {
            "{host_arch}": rctx.attr.host_arch,
            "{host_os}": rctx.attr.host_os,
            "{toolchain}": rctx.attr.toolchain,
        },
    )

risc0_toolchain_proxy = repository_rule(
    attrs = {
        "host_arch": attr.string(mandatory = True),
        "host_os": attr.string(mandatory = True),
        "toolchain": attr.string(mandatory = True),
    },
    implementation = _proxy_impl,
)

def risc0_toolchain(name, version):
    toolchain_names = []
    for toolchain in TOOLCHAINS:
        repo_name = "{name}_{suffix}".format(name = name, suffix = toolchain.archive)
        risc0_toolchain_repo(
            name = repo_name,
            version = version,
            archive = toolchain.archive,
            sha256 = toolchain.sha256,
        )

        proxy_name = repo_name + "_toolchain"
        risc0_toolchain_proxy(
            name = proxy_name,
            host_arch = toolchain.host_arch,
            host_os = toolchain.host_os,
            toolchain = "@{repo_name}//:cc-toolchain".format(repo_name = repo_name),
        )

        toolchain_names.append("@{name}//:toolchain".format(name = proxy_name))
    native.register_toolchains(*toolchain_names)
