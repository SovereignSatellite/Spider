from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path


BENCHMARKS_DIRECTORY = Path(__file__).resolve().parents[1]
REPOSITORY_DIRECTORY = BENCHMARKS_DIRECTORY.parent
CASES_DIRECTORY = BENCHMARKS_DIRECTORY / "cases"
HARNESS_DIRECTORY = BENCHMARKS_DIRECTORY / "harness"
BUILD_DIRECTORY = REPOSITORY_DIRECTORY / "target" / "benchmarks"
FIGURES_DIRECTORY = REPOSITORY_DIRECTORY / "target" / "figures"


@dataclass(frozen=True)
class RuntimePreset:
    optimization: str
    native_mode: str
    native_arguments: tuple[str, ...]


@dataclass(frozen=True)
class TargetConfiguration:
    name: str
    compiler_target: str
    runtime_executable: str
    runtime_arguments: tuple[str, ...]
    program_arguments_prefix: tuple[str, ...]
    translated_harness_by_result_type: dict[str, str]
    translated_file_extension: str
    display_name: str
    accelerator_name: str
    chart_color: str
    runtime_presets: tuple[RuntimePreset, ...]

    def runtime_command_arguments(self, preset: RuntimePreset) -> tuple[str, ...]:
        command_arguments = self.runtime_arguments + preset.native_arguments
        return tuple(
            argument.format(
                optimization=preset.optimization,
                native_mode=preset.native_mode,
            )
            for argument in command_arguments
        )

    def translated_harness_path(self, harness_directory: Path, result_type: str) -> Path:
        return harness_directory / self.translated_harness_by_result_type[result_type]

    def translated_program_path(self, build_directory: Path, benchmark_name: str) -> Path:
        return build_directory / (
            f"{benchmark_name}.{self.name}.{self.translated_file_extension}"
        )


@dataclass(frozen=True)
class Benchmark:
    directory: Path
    strength: int
    result_type: str
    expected_result: int | float

    @property
    def name(self) -> str:
        return self.directory.name

    @property
    def lua_source(self) -> Path:
        return self.directory / "lua" / "source.lua"

    @property
    def rust_source(self) -> Path:
        return self.directory / "rust" / "source.rs"


TARGET_CONFIGURATIONS = (
    TargetConfiguration(
        name="luau",
        compiler_target="luau",
        runtime_executable="luau",
        runtime_arguments=("-{optimization}",),
        program_arguments_prefix=("--program-args",),
        translated_harness_by_result_type={
            "i32": "translated.lua",
            "f64": "translated.lua",
        },
        translated_file_extension="lua",
        display_name="Luau",
        accelerator_name="NCG",
        chart_color="#00A2FF",
        runtime_presets=(
            RuntimePreset("O0", "off", ()),
            RuntimePreset("O0", "on", ("--codegen",)),
            RuntimePreset("O2", "off", ()),
            RuntimePreset("O2", "on", ("--codegen",)),
        ),
    ),
    TargetConfiguration(
        name="luajit",
        compiler_target="lua-jit",
        runtime_executable="luajit",
        runtime_arguments=("-{optimization}",),
        program_arguments_prefix=(),
        translated_harness_by_result_type={
            "i32": "translated.lua",
            "f64": "translated-f64.lua",
        },
        translated_file_extension="lua",
        display_name="Lua",
        accelerator_name="JIT",
        chart_color="#000080",
        runtime_presets=(
            RuntimePreset("O0", "off", ("-joff",)),
            RuntimePreset("O0", "on", ("-jon",)),
            RuntimePreset("O3", "off", ("-joff",)),
            RuntimePreset("O3", "on", ("-jon",)),
        ),
    ),
)


def load_benchmark(manifest_path: Path) -> Benchmark:
    with manifest_path.open(encoding="utf-8") as manifest_file:
        manifest = json.load(manifest_file)

    if set(manifest) != {"strength", "type", "expected"}:
        raise ValueError(f"{manifest_path}: expected strength, type, and expected")

    result_type = manifest["type"]
    expected_result = manifest["expected"]
    if result_type == "i32":
        if (
            not isinstance(expected_result, int)
            or isinstance(expected_result, bool)
            or not -0x8000_0000 <= expected_result <= 0x7FFF_FFFF
        ):
            raise ValueError(f"{manifest_path}: expected must fit a signed i32")
    elif result_type == "f64":
        if (
            not isinstance(expected_result, (int, float))
            or isinstance(expected_result, bool)
            or not math.isfinite(expected_result)
        ):
            raise ValueError(f"{manifest_path}: expected must be a finite number")
        expected_result = float(expected_result)
    else:
        raise ValueError(f"{manifest_path}: type must be i32 or f64")

    strength = manifest["strength"]
    if (
        not isinstance(strength, int)
        or isinstance(strength, bool)
        or not 1 <= strength <= 0xFFFF_FFFF
    ):
        raise ValueError(f"{manifest_path}: strength must be a positive u32")

    return Benchmark(manifest_path.parent, strength, result_type, expected_result)


def load_suite() -> tuple[Benchmark, ...]:
    benchmarks = tuple(
        load_benchmark(manifest_path)
        for manifest_path in sorted(CASES_DIRECTORY.glob("*/benchmark.json"))
    )
    if not benchmarks:
        raise ValueError(f"no benchmarks found under {CASES_DIRECTORY}")
    return benchmarks
