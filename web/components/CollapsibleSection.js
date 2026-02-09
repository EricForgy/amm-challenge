"use client";

import { useState } from "react";

export default function CollapsibleSection({
  title,
  subtitle,
  defaultOpen = true,
  children
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section className="collapse">
      <button
        type="button"
        className="collapseHead"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <span className={`collapseChevron ${open ? "open" : ""}`}>▸</span>
        <span className="collapseTitleWrap">
          <span className="collapseTitle">{title}</span>
          {subtitle ? <span className="collapseSubtitle">{subtitle}</span> : null}
        </span>
      </button>
      {open ? <div className="collapseBody">{children}</div> : null}
    </section>
  );
}
