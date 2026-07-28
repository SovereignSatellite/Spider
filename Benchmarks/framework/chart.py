import html
import math
from collections.abc import Iterable
from itertools import groupby
from pathlib import Path

from .report import SummaryMeasurement
from .suite import TARGET_CONFIGURATIONS

RUST_COLOR = "#DEA584"


def render_chart(measurements: list[SummaryMeasurement]) -> str:
    width = 1280
    plot_left = 330
    plot_width = 850
    top = 150
    row_height = 27
    language_heading_height = 32
    language_gap = 10
    benchmark_gap = 38
    runtime_configuration_count = sum(
        len(target.runtime_presets) for target in TARGET_CONFIGURATIONS
    )
    benchmark_count = len(measurements) // runtime_configuration_count
    target_group_count = benchmark_count * len(TARGET_CONFIGURATIONS) * 2
    height = (
        top
        + len(measurements) * 2 * row_height
        + benchmark_count * benchmark_gap
        + target_group_count * (language_heading_height + language_gap)
        + 55
    )
    minimum_seconds = min(
        duration
        for measurement in measurements
        for duration in (measurement.pure_seconds, measurement.translated_seconds)
    )
    maximum_seconds = max(
        duration
        for measurement in measurements
        for duration in (measurement.pure_seconds, measurement.translated_seconds)
    )
    logarithm_minimum = math.floor(math.log10(minimum_seconds))
    logarithm_maximum = math.ceil(math.log10(maximum_seconds))
    if logarithm_minimum == logarithm_maximum:
        logarithm_maximum += 1

    def horizontal_position(seconds: float) -> float:
        return plot_left + (
            (math.log10(seconds) - logarithm_minimum)
            * plot_width
            / (logarithm_maximum - logarithm_minimum)
        )

    svg_lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#fbfaf7"/>',
        '<style>text { font-family: ui-sans-serif, system-ui, sans-serif; fill: #24292f; }</style>',
        '<text x="64" y="60" font-size="30" font-weight="700">'
        "Benchmark runtime by test and language</text>",
        '<text x="64" y="94" font-size="17" fill="#57606a">'
        "Lower is faster · paired workloads use the same configured strength · "
        "seconds on a logarithmic scale</text>",
    ]
    for exponent in range(logarithm_minimum, logarithm_maximum + 1):
        tick_seconds = 10**exponent
        tick_position = horizontal_position(tick_seconds)
        svg_lines.extend(
            (
                f'<line x1="{tick_position:.1f}" y1="118" '
                f'x2="{tick_position:.1f}" y2="{height - 25}" '
                'stroke="#d8dee4"/>',
                f'<text x="{tick_position:.1f}" y="132" font-size="14" '
                f'text-anchor="middle">{tick_seconds:g} s</text>',
            )
        )

    vertical_position = top

    def add_implementation_group(
        group_label: str,
        color: str,
        accelerator: str,
        configuration_times: Iterable[tuple[SummaryMeasurement, float]],
        vertical_position: float,
    ) -> float:
        svg_lines.append(
            f'<text x="64" y="{vertical_position + 20:.1f}" font-size="16" '
            f'font-weight="650">{group_label}</text>'
        )
        vertical_position += language_heading_height
        for measurement, seconds in configuration_times:
            center = vertical_position + row_height / 2
            seconds_position = horizontal_position(seconds)
            bar_width = max(2, seconds_position - plot_left)
            preset = measurement.preset
            configuration = (
                f"{preset.optimization} · {accelerator} {preset.native_mode}"
            )
            svg_lines.extend(
                (
                    f'<text x="{plot_left - 18}" y="{center + 5:.1f}" '
                    f'font-size="15" text-anchor="end">{configuration}</text>',
                    f'<rect x="{plot_left}" y="{center - 8:.1f}" '
                    f'width="{bar_width:.1f}" height="16" rx="4" fill="{color}"/>',
                    f'<text x="{seconds_position + 8:.1f}" y="{center + 5:.1f}" '
                    f'font-size="14" font-weight="650">{seconds:.6g} s</text>',
                )
            )
            vertical_position += row_height
        return vertical_position + language_gap

    for benchmark_name, benchmark_group in groupby(
        measurements,
        key=lambda measurement: measurement.benchmark.name,
    ):
        vertical_position += benchmark_gap
        svg_lines.append(
            f'<text x="64" y="{vertical_position - 12}" '
            f'font-size="20" font-weight="700">'
            f"{html.escape(benchmark_name, quote=True)}</text>"
        )
        for target, target_group in groupby(
            benchmark_group,
            key=lambda measurement: measurement.target,
        ):
            target_measurements = tuple(target_group)
            vertical_position = add_implementation_group(
                f"{target.display_name} (Handwritten)",
                target.chart_color,
                target.accelerator_name,
                (
                    (measurement, measurement.pure_seconds)
                    for measurement in target_measurements
                ),
                vertical_position,
            )
            vertical_position = add_implementation_group(
                f"{target.display_name} (Rust)",
                RUST_COLOR,
                target.accelerator_name,
                (
                    (measurement, measurement.translated_seconds)
                    for measurement in target_measurements
                ),
                vertical_position,
            )

    svg_lines.append("</svg>")
    return "\n".join(svg_lines) + "\n"


def write_chart(measurements: list[SummaryMeasurement], output_path: Path) -> None:
    output_path.write_text(render_chart(measurements), encoding="utf-8")
