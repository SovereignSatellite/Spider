from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .suite import Benchmark, RuntimePreset, TargetConfiguration


SUMMARY_HEADER = (
    "benchmark",
    "target",
    "runtime_optimization",
    "jit_or_ncg",
    "strength",
    "pure_seconds",
    "translated_seconds",
    "translated_over_pure",
)


@dataclass(frozen=True)
class SummaryMeasurement:
    benchmark: Benchmark
    target: TargetConfiguration
    preset: RuntimePreset
    pure_seconds: float
    translated_seconds: float

    @property
    def ratio(self) -> float:
        return self.translated_seconds / self.pure_seconds


def write_summary(summary_path: Path, measurements: list[SummaryMeasurement]) -> None:
    with summary_path.open("w", encoding="utf-8", newline="\n") as summary_file:
        summary_file.write("\t".join(SUMMARY_HEADER) + "\n")
        for measurement in measurements:
            row = (
                measurement.benchmark.name,
                measurement.target.name,
                measurement.preset.optimization,
                measurement.preset.native_mode,
                str(measurement.benchmark.strength),
                f"{measurement.pure_seconds:.9f}",
                f"{measurement.translated_seconds:.9f}",
                f"{measurement.ratio:.3f}",
            )
            summary_file.write("\t".join(row) + "\n")
