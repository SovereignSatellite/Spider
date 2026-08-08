from __future__ import annotations

import math
import subprocess
from pathlib import Path
from typing import TextIO

from .build import build_suite
from .chart import write_chart
from .report import SummaryMeasurement, write_summary
from .suite import (
    BUILD_DIRECTORY,
    FIGURES_DIRECTORY,
    Benchmark,
    RuntimePreset,
    TARGET_CONFIGURATIONS,
    TargetConfiguration,
    load_suite,
)


SAMPLE_COUNT = 3


def runtime_command(
    target: TargetConfiguration,
    preset: RuntimePreset,
    program_path: Path,
    program_arguments: tuple[str, ...],
) -> tuple[str, ...]:
    arguments = target.runtime_command_arguments(preset)
    return (
        target.runtime_executable,
        *arguments,
        str(program_path),
        *target.program_arguments_prefix,
        *program_arguments,
    )


def measure_program(
    benchmark: Benchmark,
    target: TargetConfiguration,
    preset: RuntimePreset,
    program_path: Path,
) -> tuple[float, ...]:
    output = subprocess.run(
        runtime_command(
            target,
            preset,
            program_path,
            (
                str(benchmark.strength),
                str(benchmark.expected_result),
            ),
        ),
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    samples = tuple(float(line) for line in output.splitlines())
    if len(samples) != SAMPLE_COUNT or any(
        not math.isfinite(sample) or sample <= 0 for sample in samples
    ):
        raise ValueError(f"invalid samples from {program_path}")
    return samples


def write_raw_samples(
    raw_results_file: TextIO,
    benchmark: Benchmark,
    implementation: str,
    target: TargetConfiguration,
    preset: RuntimePreset,
    samples: tuple[float, ...],
) -> None:
    prefix = "\t".join(
        (
            benchmark.name,
            implementation,
            target.name,
            preset.optimization,
            preset.native_mode,
        )
    )
    for sample in samples:
        raw_results_file.write(f"{prefix}\t{sample:.9f}\n")


def run_suite() -> None:
    benchmarks = load_suite()
    build_suite(benchmarks)
    FIGURES_DIRECTORY.mkdir(parents=True, exist_ok=True)

    summary = []
    raw_results_path = FIGURES_DIRECTORY / "matrix-results.tsv"
    with raw_results_path.open("w", encoding="utf-8", newline="\n") as raw_results_file:
        for benchmark in benchmarks:
            pure_path = BUILD_DIRECTORY / f"{benchmark.name}.pure.lua"
            for target in TARGET_CONFIGURATIONS:
                translated_path = target.translated_program_path(
                    BUILD_DIRECTORY,
                    benchmark.name,
                )
                for preset in target.runtime_presets:
                    pure_samples = measure_program(benchmark, target, preset, pure_path)
                    translated_samples = measure_program(
                        benchmark,
                        target,
                        preset,
                        translated_path,
                    )
                    write_raw_samples(
                        raw_results_file,
                        benchmark,
                        "pure",
                        target,
                        preset,
                        pure_samples,
                    )
                    write_raw_samples(
                        raw_results_file,
                        benchmark,
                        "translated",
                        target,
                        preset,
                        translated_samples,
                    )
                    summary.append(
                        SummaryMeasurement(
                            benchmark,
                            target,
                            preset,
                            math.fsum(pure_samples) / SAMPLE_COUNT,
                            math.fsum(translated_samples) / SAMPLE_COUNT,
                        )
                    )

    write_summary(FIGURES_DIRECTORY / "matrix-summary.tsv", summary)
    write_chart(summary, FIGURES_DIRECTORY / "matrix.svg")
