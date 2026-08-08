from __future__ import annotations

import os
import subprocess
from pathlib import Path

from .suite import (
    BUILD_DIRECTORY,
    HARNESS_DIRECTORY,
    REPOSITORY_DIRECTORY,
    Benchmark,
    TARGET_CONFIGURATIONS,
    TargetConfiguration,
)


RUST_OPTIMIZATION_ARGUMENTS = (
    "-C",
    "codegen-units=1",
    "-C",
    "lto=fat",
    "-C",
    "opt-level=3",
    "-C",
    "panic=abort",
    "-C",
    "strip=symbols",
)


def spider_executable_path() -> Path:
    executable_name = "spider-cli.exe" if os.name == "nt" else "spider-cli"
    return REPOSITORY_DIRECTORY / "target" / "release" / executable_name


def compile_rust_source(benchmark: Benchmark) -> Path:
    wasm_path = BUILD_DIRECTORY / f"{benchmark.name}.wasm"
    subprocess.run(
        (
            "rustc",
            "--edition",
            "2024",
            "--target",
            "wasm32-unknown-unknown",
            *RUST_OPTIMIZATION_ARGUMENTS,
            str(benchmark.rust_source),
            "-o",
            str(wasm_path),
        ),
        check=True,
        cwd=REPOSITORY_DIRECTORY,
    )
    return wasm_path


def assemble_pure_program(benchmark: Benchmark, runtime_harness: str) -> None:
    program = benchmark.lua_source.read_text(encoding="utf-8") + "\n" + runtime_harness
    (BUILD_DIRECTORY / f"{benchmark.name}.pure.lua").write_text(program, encoding="utf-8")


def compile_translated_program(
    benchmark: Benchmark,
    wasm_path: Path,
    target: TargetConfiguration,
    spider_executable: Path,
    runtime_harness: str,
) -> None:
    spider_output = subprocess.run(
        (
            str(spider_executable),
            "compile",
            "--optimization-level",
            "3",
            "--target",
            target.compiler_target,
            str(wasm_path),
        ),
        check=True,
        cwd=REPOSITORY_DIRECTORY,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    translated_harness = target.translated_harness_path(
        HARNESS_DIRECTORY,
        benchmark.result_type,
    ).read_text(encoding="utf-8")
    program = spider_output + "\n" + translated_harness + "\n" + runtime_harness
    target.translated_program_path(BUILD_DIRECTORY, benchmark.name).write_text(
        program,
        encoding="utf-8",
    )


def build_suite(benchmarks: tuple[Benchmark, ...]) -> None:
    BUILD_DIRECTORY.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        (
            "cargo",
            "build",
            "--release",
            "--package",
            "spider-cli",
            "--manifest-path",
            str(REPOSITORY_DIRECTORY / "Cargo.toml"),
            "--target-dir",
            str(REPOSITORY_DIRECTORY / "target"),
        ),
        check=True,
        cwd=REPOSITORY_DIRECTORY,
    )

    spider_executable = spider_executable_path()
    runtime_harness = (HARNESS_DIRECTORY / "runtime.lua").read_text(encoding="utf-8")
    for benchmark in benchmarks:
        wasm_path = compile_rust_source(benchmark)
        assemble_pure_program(benchmark, runtime_harness)
        for target in TARGET_CONFIGURATIONS:
            compile_translated_program(
                benchmark,
                wasm_path,
                target,
                spider_executable,
                runtime_harness,
            )
