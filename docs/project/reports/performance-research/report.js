"use strict";

(() => {
  const report = window.FDU_PERFORMANCE_RESEARCH;
  const svgNamespace = "http://www.w3.org/2000/svg";

  const byId = (id) => document.getElementById(id);

  const element = (tag, className, text) => {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  };

  const svgElement = (tag, attributes = {}) => {
    const node = document.createElementNS(svgNamespace, tag);
    for (const [name, value] of Object.entries(attributes)) {
      node.setAttribute(name, String(value));
    }
    return node;
  };

  const replaceChildren = (node, ...children) => {
    node.replaceChildren(...children);
    return node;
  };

  const signedNumber = (value, digits = 1) => {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    const number = Number(value);
    const absolute = Math.abs(number).toFixed(digits);
    if (number < 0) return `−${absolute}`;
    if (number > 0) return `+${absolute}`;
    return Number(absolute).toFixed(digits);
  };

  const formatPercent = (value, digits = 1) => `${signedNumber(value, digits)}%`;

  const formatImprovementPercent = (changePercent, digits = 1) => {
    if (changePercent === null || changePercent === undefined || Number.isNaN(Number(changePercent))) {
      return "—";
    }
    return formatPercent(-Number(changePercent), digits);
  };

  const formatInteger = (value) => {
    if (value === null || value === undefined) return "—";
    return new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 }).format(Number(value));
  };

  const formatDecimal = (value, digits = 1) => {
    if (value === null || value === undefined) return "—";
    return new Intl.NumberFormat("en-US", {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(Number(value));
  };

  const platformFromHost = (host) => {
    if (typeof host !== "string") return "Unknown";
    if (host.startsWith("Darwin")) return "macOS";
    if (host.startsWith("Linux")) return "Linux";
    return host || "Unknown";
  };

  const effectClass = (effect) => {
    if (effect === "improved") return "effect-good";
    if (effect === "regressed") return "effect-bad";
    return "effect-neutral";
  };

  const pointEffectClass = (effect) => {
    if (effect === "improved") return "effect-improved";
    if (effect === "regressed") return "effect-regressed";
    return "effect-unclear";
  };

  const decisionClass = (decision) => `decision-${String(decision).replaceAll("_", "-")}`;

  const measurementEffect = (measurement) => {
    const [low, high] = measurement.ci95_pct || [null, null];
    if (high !== null && high < 0) return "improved";
    if (low !== null && low > 0) return "regressed";
    return "unclear";
  };

  const badge = (text, className) => element("span", `effect-badge ${className}`, text);

  const decisionBadge = (decision) =>
    element(
      "span",
      `decision-badge ${decisionClass(decision)}`,
      String(decision).replaceAll("-", " "),
    );

  const formatInterval = (interval, digits = 1) => {
    if (!interval || interval[0] === null || interval[1] === null) return "not recorded";
    return `[${formatPercent(interval[0], digits)}, ${formatPercent(interval[1], digits)}]`;
  };

  const formatImprovementInterval = (interval, digits = 1) => {
    if (!interval || interval[0] === null || interval[1] === null) return "not recorded";
    return formatInterval([-Number(interval[1]), -Number(interval[0])], digits);
  };

  const formatMetricValue = (value, metric) => {
    if (value === null || value === undefined) return "—";
    if (String(metric).endsWith("_ns")) return `${formatDecimal(Number(value) / 1_000_000, 1)} ms`;
    if (String(metric).endsWith("_bytes")) {
      return `${formatDecimal(Number(value) / (1024 * 1024), 1)} MiB`;
    }
    return formatDecimal(value, 2);
  };

  const failVisible = (message) => {
    const main = byId("main");
    const notice = element("div", "callout callout-caution");
    notice.append(element("strong", "", "The generated report data could not be loaded. "));
    notice.append(document.createTextNode(message));
    main.prepend(notice);
  };

  if (!report) {
    failVisible("Regenerate report-data.js and reload this page.");
    return;
  }

  const experimentsById = new Map(report.experiments.map((experiment) => [experiment.id, experiment]));

  const initializeTheme = () => {
    const root = document.documentElement;
    const toggle = byId("theme-toggle");
    let saved = null;
    try {
      saved = window.localStorage.getItem("fdu-performance-report-theme");
    } catch (_error) {
      saved = null;
    }
    const preferred = window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    const apply = (theme) => {
      root.dataset.theme = theme;
      toggle.setAttribute("aria-label", `Use ${theme === "dark" ? "light" : "dark"} theme`);
      byId("theme-icon").textContent = theme === "dark" ? "☀" : "◐";
    };
    apply(saved === "dark" || saved === "light" ? saved : preferred);
    toggle.addEventListener("click", () => {
      const next = root.dataset.theme === "dark" ? "light" : "dark";
      apply(next);
      try {
        window.localStorage.setItem("fdu-performance-report-theme", next);
      } catch (_error) {
        // The report remains fully usable when local file storage is unavailable.
      }
    });
  };

  const comparisonBar = (label, value, maximum, effect, candidate = false) => {
    const row = element(
      "div",
      `bar-line${candidate ? ` bar-line-candidate effect-line-${effect}` : ""}`,
    );
    row.append(element("span", "", label));
    const track = element("span", "bar-track");
    const fill = element("span", "bar-fill");
    fill.style.width = `${Math.max(2, (Number(value) / maximum) * 100)}%`;
    track.append(fill);
    row.append(track, element("span", "", `${formatDecimal(value, 1)} ms`));
    return row;
  };

  const renderOutcomes = () => {
    const target = byId("outcome-grid");
    const platforms = [...new Set(report.current_outcomes.measurements.map((item) => item.platform))];
    const fragment = document.createDocumentFragment();
    for (const platform of platforms) {
      const measurements = report.current_outcomes.measurements.filter((item) => item.platform === platform);
      const article = element("article", "outcome-platform panel");
      const heading = element("div", "outcome-platform-header");
      heading.append(
        element("h3", "", platform),
        element("span", "", `${measurements.length} measured ${measurements.length === 1 ? "job" : "jobs"}`),
      );
      article.append(heading);

      for (const measurement of measurements) {
        const effect = measurementEffect(measurement);
        const row = element("div", "outcome-row");
        const line = element("div", "outcome-line");
        line.append(
          element("span", "outcome-label", measurement.label),
          badge(formatPercent(measurement.change_pct, platform === "macOS" ? 1 : 1), effectClass(effect)),
        );
        const values = element(
          "div",
          "outcome-values",
          `${formatDecimal(measurement.control_ms, 1)} → ${formatDecimal(measurement.candidate_ms, 1)} ms`,
        );
        const bars = element("div", "comparison-bars");
        const maximum = Math.max(measurement.control_ms, measurement.candidate_ms);
        bars.append(
          comparisonBar("Control", measurement.control_ms, maximum, effect, false),
          comparisonBar("Candidate", measurement.candidate_ms, maximum, effect, true),
        );
        row.append(
          line,
          values,
          bars,
          element("p", "outcome-ci", `95% interval ${formatInterval(measurement.ci95_pct, 1)}`),
        );
        if (measurement.caveat) row.append(element("p", "outcome-caveat", measurement.caveat));
        article.append(row);
      }
      fragment.append(article);
    }
    replaceChildren(target, fragment);
  };

  const setupComposite = () => {
    const linuxInput = byId("linux-weight");
    const warmInput = byId("warm-weight");
    const score = byId("composite-score");
    const scorePanel = score.closest(".composite-score");
    const fill = byId("composite-fill");
    const cellsTarget = byId("composite-cells");

    const update = () => {
      const linuxShare = Number(linuxInput.value) / 100;
      const warmShare = Number(warmInput.value) / 100;
      byId("linux-weight-output").textContent = `${linuxInput.value}% Linux`;
      byId("warm-weight-output").textContent = `${warmInput.value}% warm`;

      let weightedLogRatio = 0;
      const renderedCells = [];
      for (const cell of report.current_outcomes.composite.cells) {
        const platformShare = cell.platform === "Linux" ? linuxShare : 1 - linuxShare;
        const stateShare = cell.job.startsWith("warm") ? warmShare : 1 - warmShare;
        const weight = platformShare * stateShare;
        weightedLogRatio += weight * Math.log1p(Number(cell.change_pct) / 100);

        const sourceMeasurement = report.current_outcomes.measurements.find(
          (measurement) => measurement.platform === cell.platform && measurement.job === cell.job,
        );
        const effect = sourceMeasurement ? measurementEffect(sourceMeasurement) : "unclear";
        const row = element("div", "composite-cell");
        const label = element("span");
        label.append(
          document.createTextNode(`${cell.platform} · ${cell.job.startsWith("warm") ? "warm" : "cold"}`),
          element("small", "", `${formatPercent(cell.change_pct, 1)} measured latency`),
        );
        row.append(label, element("strong", effectClass(effect), `${formatDecimal(weight * 100, 0)}% weight`));
        renderedCells.push(row);
      }

      const ratio = Math.exp(weightedLogRatio);
      const reduction = (1 - ratio) * 100;
      const throughput = (1 / ratio - 1) * 100;
      const favorable = reduction >= 0;
      score.textContent = favorable
        ? `${formatDecimal(reduction, 1)}% lower`
        : `${formatDecimal(Math.abs(reduction), 1)}% higher`;
      scorePanel.classList.toggle("is-bad", !favorable);
      byId("composite-throughput").textContent = `${formatPercent(throughput, 1)} throughput equivalent`;
      fill.style.width = `${Math.min(100, Math.abs(reduction) / 35 * 100)}%`;
      replaceChildren(cellsTarget, ...renderedCells);
    };

    linuxInput.addEventListener("input", update);
    warmInput.addEventListener("input", update);
    update();
  };

  const addDefinition = (list, term, value) => {
    const wrapper = element("div");
    wrapper.append(element("dt", "", term), element("dd", "", value));
    list.append(wrapper);
  };

  const renderExperimentDetail = (experiment) => {
    const target = byId("experiment-detail");
    const header = element("div", "detail-header");
    const identity = element("div");
    identity.append(
      element(
        "p",
        "detail-kicker",
        `${experiment.id} · ${experiment.platform} · ${experiment.date}`,
      ),
      element("h3", "", experiment.title),
    );
    const badges = element("div", "detail-badges");
    badges.append(
      decisionBadge(experiment.verdict.decision),
      badge(
        `${formatImprovementPercent(experiment.primary.change_pct, 2)} improvement`,
        effectClass(experiment.effect),
      ),
    );
    header.append(identity, badges);

    const facts = element("dl", "detail-grid");
    addDefinition(facts, "Primary job", experiment.primary.job || "not recorded");
    addDefinition(facts, "Metric", experiment.primary.metric || "not recorded");
    addDefinition(facts, "Trial pairs", formatInteger(experiment.primary.pairs || experiment.method.trials));
    addDefinition(facts, "Tree", `${formatInteger(experiment.subject.tree_entries)} entries`);

    const columns = element("div", "detail-columns");
    const setup = element("div");
    setup.append(
      element("h4", "", "Setup"),
      element("p", "", `Control: ${experiment.method.control}`),
      element("p", "", `Candidate: ${experiment.method.candidate}`),
      element(
        "p",
        "",
        `Medians: ${formatMetricValue(experiment.primary.control_median, experiment.primary.metric)} → ${formatMetricValue(experiment.primary.candidate_median, experiment.primary.metric)}. Relative improvement: ${formatImprovementPercent(experiment.primary.change_pct, 2)}; 95% interval ${formatImprovementInterval(experiment.primary.ci95_pct, 2)}.`,
      ),
    );
    const verdict = element("div");
    verdict.append(
      element("h4", "", "Verdict"),
      element("p", "detail-reason", experiment.verdict.reason),
      element(
        "p",
        "",
        `Hypotheses: ${experiment.hypotheses.length ? experiment.hypotheses.join(", ") : "none recorded"}. Complexity: ${formatInteger(experiment.complexity.lines_changed)} production lines, ${experiment.complexity.new_dependencies.length} new dependencies, ${formatInteger(experiment.complexity.new_unsafe_blocks)} new unsafe blocks.`,
      ),
    );
    const source = element("a", "detail-link", "Open the complete experiment artifact →");
    source.href = experiment.source;
    verdict.append(source);
    columns.append(setup, verdict);
    replaceChildren(target, header, facts, columns);
  };

  const setupExperimentPlot = () => {
    const svg = byId("experiment-plot");
    const platformFilter = byId("plot-platform");
    const phaseFilter = byId("plot-phase");
    const effectFilter = byId("plot-effect");
    const width = 1240;
    const height = 760;
    const margin = { right: 24, left: 82 };
    const innerWidth = width - margin.left - margin.right;
    const experimentCount = report.experiments.length;
    const absolutePanel = { top: 72, bottom: 315 };
    const relativePanel = { top: 400, bottom: 690 };
    const improvementValues = report.experiments
      .map((experiment) => experiment.primary.change_pct)
      .filter((value) => Number.isFinite(value))
      .map((value) => -Number(value));
    const maximumMagnitude =
      Math.ceil((Math.max(3, ...improvementValues.map((value) => Math.abs(value))) * 1.05) / 10) * 10;
    const transformedLimit = Math.sqrt(maximumMagnitude);
    const durationExperiments = report.experiments.filter(
      (experiment) =>
        String(experiment.primary.metric).endsWith("_ns") &&
        Number(experiment.primary.control_median) > 0 &&
        Number(experiment.primary.candidate_median) > 0,
    );
    const durationValues = durationExperiments.flatMap((experiment) => [
      Number(experiment.primary.control_median) / 1_000_000,
      Number(experiment.primary.candidate_median) / 1_000_000,
    ]);
    const observedDurationLogs = durationValues.map((value) => Math.log10(value));
    const observedLogMinimum = observedDurationLogs.length ? Math.min(...observedDurationLogs) : 0;
    const observedLogMaximum = observedDurationLogs.length ? Math.max(...observedDurationLogs) : 3;
    const logPadding = Math.max(0.08, (observedLogMaximum - observedLogMinimum) * 0.06);
    const durationLogMinimum = observedLogMinimum - logPadding;
    const durationLogMaximum = observedLogMaximum + logPadding;
    let pinned = experimentsById.get("exp-054") || report.experiments.at(-1);

    for (const phase of report.phases) {
      const option = element("option", "", phase.label);
      option.value = phase.id;
      phaseFilter.append(option);
    }

    const x = (number) => margin.left + ((number + 0.5) / experimentCount) * innerWidth;
    const transformEffect = (value) => Math.sign(value) * Math.sqrt(Math.abs(value));
    const relativeY = (improvement) =>
      relativePanel.top +
      ((transformedLimit - transformEffect(improvement)) / (2 * transformedLimit)) *
        (relativePanel.bottom - relativePanel.top);
    const absoluteY = (milliseconds) =>
      absolutePanel.top +
      ((Math.log10(milliseconds) - durationLogMinimum) /
        (durationLogMaximum - durationLogMinimum)) *
        (absolutePanel.bottom - absolutePanel.top);
    const isDurationExperiment = (experiment) => durationExperiments.includes(experiment);
    const improvementFor = (experiment) => -Number(experiment.primary.change_pct);
    const formatDurationTick = (value) =>
      new Intl.NumberFormat("en-US", {
        maximumFractionDigits: value < 10 ? 1 : 0,
      }).format(value);

    const durationTicks = [];
    for (
      let exponent = Math.floor(durationLogMinimum);
      exponent <= Math.ceil(durationLogMaximum);
      exponent += 1
    ) {
      for (const multiplier of [1, 2, 5]) {
        const value = multiplier * 10 ** exponent;
        const valueLog = Math.log10(value);
        if (valueLog >= durationLogMinimum && valueLog <= durationLogMaximum) {
          durationTicks.push(value);
        }
      }
    }

    const relativeMagnitudes = [3, 10, 25, 50, 100, 200, 500].filter(
      (value) => value < maximumMagnitude,
    );
    relativeMagnitudes.push(maximumMagnitude);
    const relativeTicks = [
      ...relativeMagnitudes.map((value) => -value).reverse(),
      0,
      ...relativeMagnitudes,
    ];

    const visibleExperiments = () =>
      report.experiments.filter((experiment) => {
        const platformMatches = platformFilter.value === "all" || experiment.platform === platformFilter.value;
        const phaseMatches = phaseFilter.value === "all" || experiment.phase === phaseFilter.value;
        const selectedEffect = effectFilter.value;
        const effectMatches =
          selectedEffect === "all" ||
          experiment.effect === selectedEffect ||
          (selectedEffect === "unclear" && ["unknown", "unclear"].includes(experiment.effect));
        return platformMatches && phaseMatches && effectMatches;
      });

    const markSelected = (id) => {
      for (const point of svg.querySelectorAll("[data-experiment-id]")) {
        point.classList.toggle("is-selected", point.dataset.experimentId === id);
      }
    };

    const showTemporary = (experiment) => {
      renderExperimentDetail(experiment);
      markSelected(experiment.id);
    };

    const restorePinned = () => {
      if (pinned) {
        renderExperimentDetail(pinned);
        markSelected(pinned.id);
      }
    };

    const attachInteraction = (group, experiment) => {
      group.addEventListener("mouseenter", () => showTemporary(experiment));
      group.addEventListener("mouseleave", restorePinned);
      group.addEventListener("focus", () => showTemporary(experiment));
      group.addEventListener("blur", restorePinned);
      group.addEventListener("click", () => {
        pinned = experiment;
        restorePinned();
      });
      group.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          pinned = experiment;
          restorePinned();
        }
      });
    };

    const appendPlatformMarker = (group, experiment, markerX, markerY, className, size) => {
      if (experiment.platform === "Linux") {
        group.append(
          svgElement("polygon", {
            points: `${markerX},${markerY - size} ${markerX + size},${markerY} ${markerX},${markerY + size} ${markerX - size},${markerY}`,
            class: className,
          }),
        );
      } else {
        group.append(
          svgElement("circle", {
            cx: markerX,
            cy: markerY,
            r: size * 0.78,
            class: className,
          }),
        );
      }
    };

    const appendPhaseBands = (panel) => {
      for (const [index, phase] of report.phases.entries()) {
        const start = margin.left + (phase.first / experimentCount) * innerWidth;
        const end = margin.left + ((phase.last + 1) / experimentCount) * innerWidth;
        svg.append(
          svgElement("rect", {
            x: start,
            y: panel.top,
            width: end - start,
            height: panel.bottom - panel.top,
            class: index % 2 === 0 ? "plot-phase-a" : "plot-phase-b",
          }),
        );
      }
    };

    const appendPanelTitle = (text, panel) => {
      const title = svgElement("text", {
        x: margin.left,
        y: panel.top - 14,
        class: "plot-panel-title",
      });
      title.textContent = text;
      svg.append(title);
    };

    const appendFrame = (panel) => {
      svg.append(
        svgElement("rect", {
          x: margin.left,
          y: panel.top,
          width: innerWidth,
          height: panel.bottom - panel.top,
          class: "plot-frame",
        }),
      );
    };

    const appendYAxisTitle = (text, panel) => {
      const center = panel.top + (panel.bottom - panel.top) / 2;
      const title = svgElement("text", {
        x: 18,
        y: center,
        transform: `rotate(-90 18 ${center})`,
        "text-anchor": "middle",
        class: "plot-label",
      });
      title.textContent = text;
      svg.append(title);
    };

    const render = () => {
      while (svg.children.length > 2) svg.removeChild(svg.lastElementChild);
      const visible = visibleExperiments();
      const visibleIds = new Set(visible.map((experiment) => experiment.id));

      appendPhaseBands(absolutePanel);
      appendPhaseBands(relativePanel);

      for (const phase of report.phases) {
        const start = margin.left + (phase.first / experimentCount) * innerWidth;
        const phaseLabel = svgElement("text", {
          x: start + 7,
          y: 27,
          class: "plot-phase-label",
        });
        phaseLabel.textContent = `${phase.first}–${phase.last}`;
        svg.append(phaseLabel);
      }

      appendPanelTitle(
        "Absolute primary runtime · control → candidate · time metrics only",
        absolutePanel,
      );
      appendPanelTitle("Relative primary-metric improvement · candidate vs control", relativePanel);

      for (const tick of durationTicks) {
        const tickY = absoluteY(tick);
        const line = svgElement("line", {
          x1: margin.left,
          y1: tickY,
          x2: width - margin.right,
          y2: tickY,
          class: "plot-grid",
        });
        const label = svgElement("text", {
          x: margin.left - 10,
          y: tickY + 4,
          "text-anchor": "end",
          class: "plot-label",
        });
        label.textContent = formatDurationTick(tick);
        svg.append(line, label);
      }

      for (const tick of relativeTicks) {
        const tickY = relativeY(tick);
        const line = svgElement("line", {
          x1: margin.left,
          y1: tickY,
          x2: width - margin.right,
          y2: tickY,
          class:
            tick === 0 ? "plot-axis" : Math.abs(tick) === 3 ? "plot-threshold" : "plot-grid",
        });
        const label = svgElement("text", {
          x: margin.left - 10,
          y: tickY + 4,
          "text-anchor": "end",
          class: "plot-label",
        });
        label.textContent = `${tick > 0 ? "+" : ""}${tick}%`;
        svg.append(line, label);
      }

      appendFrame(absolutePanel);
      appendFrame(relativePanel);
      appendYAxisTitle("Runtime (ms, log scale) · faster ↑", absolutePanel);
      appendYAxisTitle("Improvement (%) · square-root scale · better ↑", relativePanel);

      for (let number = 0; number < experimentCount; number += 5) {
        const tickX = x(number);
        const tick = svgElement("line", {
          x1: tickX,
          y1: relativePanel.bottom,
          x2: tickX,
          y2: relativePanel.bottom + 5,
          class: "plot-axis",
        });
        const label = svgElement("text", {
          x: tickX,
          y: relativePanel.bottom + 20,
          "text-anchor": "middle",
          class: "plot-label",
        });
        label.textContent = String(number).padStart(3, "0");
        svg.append(tick, label);
      }
      const xTitle = svgElement("text", {
        x: margin.left + innerWidth / 2,
        y: height - 12,
        "text-anchor": "middle",
        class: "plot-label",
      });
      xTitle.textContent = "Experiment sequence";
      svg.append(xTitle);

      for (const experiment of visible.filter(isDurationExperiment)) {
        const pointX = x(experiment.number);
        const controlX = pointX - 4;
        const candidateX = pointX + 4;
        const controlMilliseconds = Number(experiment.primary.control_median) / 1_000_000;
        const candidateMilliseconds = Number(experiment.primary.candidate_median) / 1_000_000;
        const controlY = absoluteY(controlMilliseconds);
        const candidateY = absoluteY(candidateMilliseconds);
        const improvement = improvementFor(experiment);
        const group = svgElement("g", {
          class: `absolute-point ${pointEffectClass(experiment.effect)}`,
          tabindex: "0",
          role: "button",
          "aria-label": `${experiment.id}, ${experiment.title}, control ${formatMetricValue(experiment.primary.control_median, experiment.primary.metric)}, candidate ${formatMetricValue(experiment.primary.candidate_median, experiment.primary.metric)}, ${formatPercent(improvement, 2)} relative improvement, ${experiment.verdict.decision}`,
        });
        group.dataset.experimentId = experiment.id;
        group.append(
          svgElement("line", {
            x1: controlX,
            y1: controlY,
            x2: candidateX,
            y2: candidateY,
            class: "absolute-connector",
          }),
          svgElement("line", {
            x1: controlX,
            y1: controlY,
            x2: candidateX,
            y2: candidateY,
            class: "absolute-hit",
          }),
        );
        appendPlatformMarker(
          group,
          experiment,
          controlX,
          controlY,
          "absolute-control-mark",
          5.5,
        );
        appendPlatformMarker(
          group,
          experiment,
          candidateX,
          candidateY,
          "absolute-candidate-mark",
          5.5,
        );
        const title = svgElement("title");
        title.textContent = `${experiment.id} · ${formatMetricValue(experiment.primary.control_median, experiment.primary.metric)} control → ${formatMetricValue(experiment.primary.candidate_median, experiment.primary.metric)} candidate · ${formatPercent(improvement, 2)} improvement`;
        group.append(title);
        attachInteraction(group, experiment);
        svg.append(group);
      }

      for (const platform of ["macOS", "Linux"]) {
        const points = visible
          .filter(
            (experiment) =>
              experiment.platform === platform &&
              experiment.effect === "improved" &&
              experiment.verdict.decision === "accepted" &&
              experiment.primary.change_pct !== null,
          )
          .sort((left, right) => left.number - right.number);
        if (points.length > 1) {
          const pathData = points
            .map((experiment, index) => {
              const improvement = improvementFor(experiment);
              return `${index === 0 ? "M" : "L"}${x(experiment.number).toFixed(2)},${relativeY(improvement).toFixed(2)}`;
            })
            .join(" ");
          svg.append(svgElement("path", { d: pathData, class: "plot-line-good" }));
        }
      }

      for (const experiment of visible) {
        if (experiment.primary.change_pct === null) continue;
        const improvement = improvementFor(experiment);
        const pointX = x(experiment.number);
        const pointY = relativeY(improvement);
        const group = svgElement("g", {
          class: `plot-point ${pointEffectClass(experiment.effect)}`,
          tabindex: "0",
          role: "button",
          "aria-label": `${experiment.id}, ${experiment.title}, ${formatPercent(improvement, 2)} relative improvement, ${experiment.verdict.decision}`,
        });
        group.dataset.experimentId = experiment.id;
        group.append(svgElement("circle", { cx: pointX, cy: pointY, r: 13, class: "point-hit" }));
        appendPlatformMarker(group, experiment, pointX, pointY, "point-mark", 7);
        const title = svgElement("title");
        title.textContent = `${experiment.id} · ${experiment.title} · ${formatPercent(improvement, 2)} relative improvement`;
        group.append(title);
        attachInteraction(group, experiment);
        svg.append(group);
      }

      if (!visibleIds.has(pinned?.id)) pinned = visible.at(-1) || null;
      if (pinned) restorePinned();
      const absoluteCount = visible.filter(isDurationExperiment).length;
      byId("plot-status").textContent = `${visible.length} experiments shown; ${absoluteCount} have time-based absolute runtimes.`;
    };

    platformFilter.addEventListener("change", render);
    phaseFilter.addEventListener("change", render);
    effectFilter.addEventListener("change", render);
    render();
  };

  const renderPhases = () => {
    const target = byId("phase-strip");
    const fragment = document.createDocumentFragment();
    for (const phase of report.phases) {
      const card = element("div", "phase-card");
      card.append(
        element("strong", "", phase.label),
        element("small", "", `exp-${String(phase.first).padStart(3, "0")}–${String(phase.last).padStart(3, "0")}`),
      );
      fragment.append(card);
    }
    replaceChildren(target, fragment);
  };

  const renderMechanisms = () => {
    const target = byId("mechanism-grid");
    const fragment = document.createDocumentFragment();
    for (const mechanism of report.mechanism_groups) {
      const card = element("article", "mechanism-card");
      card.append(element("h3", "", mechanism.title), element("p", "", mechanism.summary));
      const chips = element("div", "experiment-chip-row");
      for (const id of mechanism.experiment_ids) {
        const experiment = experimentsById.get(id);
        if (!experiment) continue;
        const chip = element("a", `experiment-chip ${pointEffectClass(experiment.effect)}`, id);
        chip.href = experiment.source;
        chip.title = `${experiment.title}: ${formatPercent(experiment.primary.change_pct, 2)}`;
        chips.append(chip);
      }
      card.append(chips);
      fragment.append(card);
    }
    replaceChildren(target, fragment);

    const studyTarget = byId("study-grid");
    const studies = document.createDocumentFragment();
    for (const study of report.supporting_studies) {
      const poorer = Number(study.primary_change_pct) > 0;
      const card = element("article", `study-card${poorer ? " study-bad" : ""}`);
      card.append(
        element("h4", "", study.title),
        element("strong", "study-number", formatPercent(study.primary_change_pct, 1)),
        element("p", "", study.headline),
        element("p", "", study.lesson),
      );
      const link = element("a", "", "Open evidence →");
      link.href = study.source;
      card.append(link);
      studies.append(card);
    }
    replaceChildren(studyTarget, studies);
  };

  const displayCell = (value, descriptor) => {
    if (value === null || value === undefined || value === "") return element("span", "", "—");
    if (descriptor.display === "tags" && Array.isArray(value)) {
      const tags = element("span", "tag-list");
      for (const item of value) tags.append(element("span", "tag", String(item)));
      return tags;
    }
    if (descriptor.display === "status") return decisionBadge(value);
    if (descriptor.display === "platform") return element("span", "", platformFromHost(value));
    if (descriptor.display === "integer") return element("span", "", formatInteger(value));
    if (descriptor.display === "percent") {
      const className = Number(value) < 0 ? "cell-good" : Number(value) > 0 ? "cell-bad" : "";
      return element("span", className, formatPercent(value, 1));
    }
    return element("span", "", Array.isArray(value) ? value.join(", ") : String(value));
  };

  const setupSchemaTable = () => {
    const projection = report.softschema_table;
    const meta = byId("schema-meta");
    const metaValues = [
      ["Contract", projection.contract],
      ["Envelope", projection.envelope],
      ["Compiled schema", projection.schema.source],
      ["Validated rows", `${formatInteger(projection.row_count)} · ${Object.keys(projection.status_counts).join(", ")}`],
    ];
    replaceChildren(
      meta,
      ...metaValues.map(([label, value]) => {
        const wrapper = element("div");
        wrapper.append(element("span", "", label), element("strong", "", value));
        return wrapper;
      }),
    );

    const head = byId("artifact-head");
    const body = byId("artifact-body");
    const search = byId("table-search");
    const decision = byId("table-decision");
    const platform = byId("table-platform");
    let sortPath = "id";
    let sortDirection = 1;

    const decisions = [
      ...new Set(projection.rows.map((row) => row.values["verdict.decision"]).filter(Boolean)),
    ].sort();
    for (const value of decisions) {
      const option = element("option", "", String(value).replaceAll("-", " "));
      option.value = value;
      decision.append(option);
    }

    const comparable = (value) => {
      if (value === null || value === undefined) return "";
      if (Array.isArray(value)) return value.join(" ").toLocaleLowerCase();
      if (typeof value === "number") return value;
      return String(value).toLocaleLowerCase();
    };

    const filteredRows = () => {
      const query = search.value.trim().toLocaleLowerCase();
      const rows = projection.rows.filter((row) => {
        const decisionMatches =
          decision.value === "all" || row.values["verdict.decision"] === decision.value;
        const rowPlatform = platformFromHost(row.values["subject.host_system"]);
        const platformMatches = platform.value === "all" || rowPlatform === platform.value;
        const queryMatches = !query || JSON.stringify(row.values).toLocaleLowerCase().includes(query);
        return decisionMatches && platformMatches && queryMatches;
      });
      rows.sort((left, right) => {
        const a = comparable(left.values[sortPath]);
        const b = comparable(right.values[sortPath]);
        if (typeof a === "number" && typeof b === "number") return (a - b) * sortDirection;
        return String(a).localeCompare(String(b), undefined, { numeric: true }) * sortDirection;
      });
      return rows;
    };

    const renderHead = () => {
      const cells = projection.columns.map((descriptor) => {
        const th = element("th");
        const active = descriptor.path === sortPath;
        th.setAttribute("aria-sort", active ? (sortDirection > 0 ? "ascending" : "descending") : "none");
        const button = element(
          "button",
          "sort-button",
          `${descriptor.label}${active ? (sortDirection > 0 ? " ↑" : " ↓") : ""}`,
        );
        button.type = "button";
        button.title = descriptor.description || `Sort by ${descriptor.label}`;
        button.addEventListener("click", () => {
          if (sortPath === descriptor.path) sortDirection *= -1;
          else {
            sortPath = descriptor.path;
            sortDirection = 1;
          }
          renderHead();
          renderBody();
        });
        th.append(button);
        return th;
      });
      replaceChildren(head, ...cells);
    };

    const renderBody = () => {
      const rows = filteredRows();
      const fragment = document.createDocumentFragment();
      for (const row of rows) {
        const tr = element("tr");
        for (const descriptor of projection.columns) {
          const td = element("td");
          const value = row.values[descriptor.path];
          if (descriptor.path === "id") {
            const inspect = element("button", "inspect-record", String(value || row.identity));
            inspect.type = "button";
            inspect.addEventListener("click", () => renderSchemaRecord(row));
            td.append(inspect);
          } else {
            td.append(displayCell(value, descriptor));
          }
          tr.append(td);
        }
        fragment.append(tr);
      }
      replaceChildren(body, fragment);
      byId("table-count").textContent = `${formatInteger(rows.length)} of ${formatInteger(projection.row_count)} artifacts`;
    };

    const renderSchemaRecord = (row) => {
      const target = byId("record-detail");
      const header = element("div", "detail-header");
      const identity = element("div");
      identity.append(
        element("p", "detail-kicker", `${projection.contract} · ${row.source}`),
        element("h3", "", String(row.values.title || row.identity)),
      );
      const link = element("a", "btn", "Open source artifact");
      link.href = row.href;
      header.append(identity, link);

      const values = element("div", "record-values");
      values.append(element("h4", "", "Selected schema fields"));
      const list = element("dl");
      for (const descriptor of projection.columns) {
        list.append(
          element("dt", "", descriptor.label),
          element(
            "dd",
            "",
            Array.isArray(row.values[descriptor.path])
              ? row.values[descriptor.path].join(", ")
              : row.values[descriptor.path] === null || row.values[descriptor.path] === undefined
                ? "—"
                : String(row.values[descriptor.path]),
          ),
        );
      }
      values.append(list);

      const raw = element("details", "raw-payload");
      raw.append(
        element("summary", "", "Structured payload"),
        element("pre", "", JSON.stringify(row.payload, null, 2)),
      );
      replaceChildren(target, header, values, raw);
    };

    search.addEventListener("input", renderBody);
    decision.addEventListener("change", renderBody);
    platform.addEventListener("change", renderBody);
    renderHead();
    renderBody();
  };

  const renderSources = () => {
    const fragment = document.createDocumentFragment();
    for (const source of report.sources) {
      const item = element("li");
      const link = element("a", "", source.label);
      link.href = source.href;
      item.append(link, element("span", "", source.role));
      fragment.append(item);
    }
    replaceChildren(byId("source-list"), fragment);
  };

  initializeTheme();
  renderOutcomes();
  setupComposite();
  setupExperimentPlot();
  renderPhases();
  renderMechanisms();
  setupSchemaTable();
  renderSources();
})();
