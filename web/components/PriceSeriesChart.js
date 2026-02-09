"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import * as d3 from "d3";

const PALETTE = [
  "#5ce0cd",
  "#7fb3ff",
  "#ffd166",
  "#ff8fab",
  "#b993ff",
  "#6ee7b7",
  "#fda4af",
  "#a5b4fc"
];

export default function PriceSeriesChart({ series }) {
  const svgRef = useRef(null);
  const [mode, setMode] = useState("price");

  const data = useMemo(() => {
    if (!Array.isArray(series) || series.length === 0) return null;
    const tokenCount = Array.isArray(series[0]?.tokens) ? series[0].tokens.length : 0;
    if (tokenCount === 0) return null;
    const tokenIndices = Array.from({ length: tokenCount }, (_, i) => i);
    return {
      tokenCount,
      series,
      tokenIndices,
      plottedTokens: tokenIndices
    };
  }, [series]);

  useEffect(() => {
    if (!data || !svgRef.current) return;

    const width = 980;
    const height = 340;
    const margin = { top: 20, right: 28, bottom: 34, left: 54 };
    const innerW = width - margin.left - margin.right;
    const innerH = height - margin.top - margin.bottom;

    const firstRow = data.series[0];
    const tokenBaselines = data.tokenIndices.map((token) => firstRow.tokens[token].p50 || 1);
    const transform = (token, value) => {
      if (mode === "return") {
        return ((value / Math.max(1e-9, tokenBaselines[token])) - 1) * 100;
      }
      return value;
    };

    const flatLow = data.series.flatMap((d) =>
      data.plottedTokens.map((t) => transform(t, d.tokens[t].p10))
    );
    const flatHigh = data.series.flatMap((d) =>
      data.plottedTokens.map((t) => transform(t, d.tokens[t].p90))
    );
    let yMin = d3.min(flatLow) ?? 0;
    let yMax = d3.max(flatHigh) ?? 1;
    if (mode === "price") {
      yMin = 0;
      yMax = Math.max(1, yMax * 1.02);
    } else {
      const span = Math.max(1e-6, yMax - yMin);
      yMin = yMin - span * 0.08;
      yMax = yMax + span * 0.08;
    }
    const maxStep = d3.max(data.series, (d) => d.step) ?? 1;

    const x = d3.scaleLinear().domain([0, maxStep]).range([0, innerW]);
    const y = d3
      .scaleLinear()
      .domain([yMin * 0.98, yMax * 1.02])
      .nice()
      .range([innerH, 0]);

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();
    svg.attr("viewBox", `0 0 ${width} ${height}`);

    const g = svg.append("g").attr("transform", `translate(${margin.left},${margin.top})`);

    g.append("g")
      .attr("transform", `translate(0,${innerH})`)
      .call(d3.axisBottom(x).ticks(8).tickSizeOuter(0))
      .call((axis) => axis.selectAll("text").attr("fill", "#9eb4d4").attr("font-size", 11))
      .call((axis) => axis.selectAll("line,path").attr("stroke", "#2b3c5e"));

    const yFormatter =
      mode === "return"
        ? (v) => `${v.toFixed(1)}%`
        : (v) => d3.format("~g")(v);
    g.append("g")
      .call(d3.axisLeft(y).ticks(6).tickSizeOuter(0).tickFormat(yFormatter))
      .call((axis) => axis.selectAll("text").attr("fill", "#9eb4d4").attr("font-size", 11))
      .call((axis) => axis.selectAll("line,path").attr("stroke", "#2b3c5e"));

    g.append("g")
      .selectAll("line")
      .data(y.ticks(6))
      .enter()
      .append("line")
      .attr("x1", 0)
      .attr("x2", innerW)
      .attr("y1", (d) => y(d))
      .attr("y2", (d) => y(d))
      .attr("stroke", "#1a2740")
      .attr("stroke-width", 1);

    for (const token of data.plottedTokens) {
      const lineData = data.series.map((d) => ({
        step: d.step,
        p10: transform(token, d.tokens[token].p10),
        p50: transform(token, d.tokens[token].p50),
        p90: transform(token, d.tokens[token].p90)
      }));
      const area = d3
        .area()
        .x((d) => x(d.step))
        .y0((d) => y(d.p10))
        .y1((d) => y(d.p90))
        .curve(d3.curveMonotoneX);
      const line = d3
        .line()
        .x((d) => x(d.step))
        .y((d) => y(d.p50))
        .curve(d3.curveMonotoneX);

      g.append("path")
        .datum(lineData)
        .attr("fill", PALETTE[token % PALETTE.length])
        .attr("fill-opacity", 0.14)
        .attr("stroke", "none")
        .attr("d", area);

      g.append("path")
        .datum(lineData)
        .attr("fill", "none")
        .attr("stroke", PALETTE[token % PALETTE.length])
        .attr("stroke-width", 2)
        .attr("d", line);
    }
  }, [data, mode]);

  if (!data) return null;

  return (
    <section className="chartPanel">
      <div className="chartModeRow">
        <button
          type="button"
          className={`chartModeBtn ${mode === "price" ? "active" : ""}`}
          onClick={() => setMode("price")}
        >
          Price
        </button>
        <button
          type="button"
          className={`chartModeBtn ${mode === "return" ? "active" : ""}`}
          onClick={() => setMode("return")}
        >
          Return %
        </button>
      </div>
      <div className="legendRow">
        {data.tokenIndices.map((i) => (
          <span key={i} className="legendItem">
            <i style={{ background: PALETTE[i % PALETTE.length] }} />
            Token {i}
          </span>
        ))}
      </div>
      <div className="chartWrap">
        <svg ref={svgRef} className="priceChart" />
      </div>
    </section>
  );
}
