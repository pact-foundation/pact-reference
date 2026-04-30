<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="html" encoding="UTF-8" indent="yes"/>

  <!-- ============================================================
       ROOT
       ============================================================ -->
  <xsl:template match="/report">
    <xsl:text disable-output-escaping="yes">&lt;!DOCTYPE html&gt;&#10;</xsl:text>
    <html lang="en">
      <head>
        <meta charset="UTF-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <title>Pact Verification Report &#8212; <xsl:value-of select="provider"/></title>
        <style>
/* ============================================================
   PACT VERIFICATION REPORT — light, editorial, print-first
   ============================================================ */

:root {
  --pass:             #166534;
  --pass-bg:          #f0fdf4;
  --pass-border:      #86efac;
  --fail:             #991b1b;
  --fail-bg:          #fff5f5;
  --fail-border:      #fca5a5;
  --pending:          #78350f;
  --pending-bg:       #fffbeb;
  --pending-border:   #fcd34d;
  --info:             #1d4ed8;
  --info-bg:          #eff6ff;
  --info-border:      #bfdbfe;
  --accent:           #e11d48;
  --text:             #1c1917;
  --text-secondary:   #57534e;
  --text-muted:       #a8a29e;
  --border:           #e7e5e4;
  --surface:          #ffffff;
  --bg:               #f9f8f6;
  --font-serif:       'Palatino Linotype', Palatino, 'Book Antiqua', Georgia, serif;
  --font-sans:        -apple-system, BlinkMacSystemFont, 'Helvetica Neue', Arial, sans-serif;
  --font-mono:        'Courier New', Courier, 'Lucida Console', monospace;
  --radius:           6px;
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: var(--font-sans);
  background: var(--bg);
  color: var(--text);
  line-height: 1.6;
  font-size: 15px;
  -webkit-font-smoothing: antialiased;
}

/* ── Header ──────────────────────────────────────── */
.report-header {
  background: var(--surface);
  border-top: 5px solid var(--accent);
  border-bottom: 1px solid var(--border);
  margin-bottom: 2rem;
}
.report-header-inner {
  max-width: 1060px;
  margin: 0 auto;
  padding: 1.75rem 1.5rem 1.5rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}
.report-meta {
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--accent);
  margin-bottom: 0.35rem;
}
.report-title {
  font-family: var(--font-serif);
  font-size: 1.875rem;
  font-weight: 700;
  color: var(--text);
  line-height: 1.2;
  letter-spacing: -0.02em;
}
.report-subtitle {
  margin-top: 0.3rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

/* ── Status stamp ────────────────────────────────── */
.status-stamp {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 0.75rem 1.5rem;
  border: 2px solid;
  border-radius: var(--radius);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  flex-shrink: 0;
  min-width: 6.5rem;
  text-align: center;
}
.status-stamp.pass {
  color: var(--pass);
  border-color: var(--pass-border);
  background: var(--pass-bg);
}
.status-stamp.fail {
  color: var(--fail);
  border-color: var(--fail-border);
  background: var(--fail-bg);
}
.stamp-icon { font-size: 1.35rem; line-height: 1; margin-bottom: 0.15rem; }
.stamp-text { font-size: 0.75rem; }

/* ── Page layout ─────────────────────────────────── */
.page-wrap {
  max-width: 1060px;
  margin: 0 auto;
  padding: 0 1.5rem 2.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

/* ── Section headings ────────────────────────────── */
section h2 {
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border);
  margin-bottom: 1rem;
}

/* ── Summary stat cards ──────────────────────────── */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  overflow: hidden;
}
.stat-card {
  padding: 1.4rem 1.25rem 1.6rem;
  text-align: center;
  position: relative;
}
.stat-card + .stat-card { border-left: 1px solid var(--border); }
.stat-card::after {
  content: '';
  display: block;
  height: 3px;
  position: absolute;
  bottom: 0; left: 0; right: 0;
  background: var(--border);
}
.stat-card.pass::after    { background: var(--pass-border); }
.stat-card.fail::after    { background: var(--fail-border); }
.stat-card.pending::after { background: var(--pending-border); }
.stat-value {
  font-family: var(--font-serif);
  font-size: 2.5rem;
  font-weight: 700;
  line-height: 1;
  margin-bottom: 0.3rem;
  color: var(--text);
}
.stat-card.pass    .stat-value { color: var(--pass); }
.stat-card.fail    .stat-value { color: var(--fail); }
.stat-card.pending .stat-value { color: var(--pending); }
.stat-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-muted);
}

/* ── Notice cards ────────────────────────────────── */
.notice {
  display: flex;
  gap: 0.7rem;
  align-items: flex-start;
  background: var(--info-bg);
  border: 1px solid var(--info-border);
  border-left: 3px solid var(--info);
  border-radius: var(--radius);
  padding: 0.7rem 1rem;
  font-size: 0.875rem;
  color: var(--info);
}
.notice + .notice { margin-top: 0.5rem; }
.notice-icon { flex-shrink: 0; margin-top: 0.05rem; }

/* ── Error / Pending cards ───────────────────────── */
.error-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  margin-bottom: 0.75rem;
  page-break-inside: avoid;
  break-inside: avoid;
}
.error-card:last-child { margin-bottom: 0; }
.error-card.fail    { border-left: 4px solid var(--fail-border); }
.error-card.pending { border-left: 4px solid var(--pending-border); }

.error-card-head {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
}
.error-card.fail    .error-card-head { background: var(--fail-bg); }
.error-card.pending .error-card-head { background: var(--pending-bg); }

.error-kind-tag {
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 0.2em 0.55em;
  border-radius: 3px;
  flex-shrink: 0;
}
.error-card.fail    .error-kind-tag { background: var(--fail-border);    color: var(--fail); }
.error-card.pending .error-kind-tag { background: var(--pending-border); color: var(--pending); }

.error-card-title {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--text);
  word-break: break-all;
}
.error-card-body { padding: 1rem; }

/* ── Error message (type=error) ──────────────────── */
.error-msg {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  background: var(--fail-bg);
  border: 1px solid var(--fail-border);
  border-radius: 4px;
  padding: 0.65rem 0.9rem;
  color: var(--fail);
  word-break: break-word;
  white-space: pre-wrap;
}
.error-card.pending .error-msg {
  background: var(--pending-bg);
  border-color: var(--pending-border);
  color: var(--pending);
}

/* ── Mismatch detail table ───────────────────────── */
.mismatch-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
  table-layout: fixed;
}
.mismatch-table th {
  text-align: left;
  padding: 0.4rem 0.75rem;
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-muted);
  background: #fafaf9;
  border-bottom: 1px solid var(--border);
}
.mismatch-table td {
  padding: 0.5rem 0.75rem;
  vertical-align: top;
  border-bottom: 1px solid var(--border);
}
.mismatch-table tr:last-child td { border-bottom: none; }
.mismatch-table tr:nth-child(even) td { background: #fafaf9; }
.col-type     { width: 7rem; }
.col-location { width: 10rem; }

/* ── Type chip ───────────────────────────────────── */
.type-chip {
  display: inline-block;
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 0.2em 0.5em;
  border-radius: 3px;
  white-space: nowrap;
}
.error-card.fail    .type-chip { background: var(--fail-bg);    color: var(--fail);    border: 1px solid var(--fail-border); }
.error-card.pending .type-chip { background: var(--pending-bg); color: var(--pending); border: 1px solid var(--pending-border); }

/* ── Inline code ─────────────────────────────────── */
code {
  font-family: var(--font-mono);
  font-size: 0.8em;
  background: #f5f5f4;
  padding: 0.1em 0.35em;
  border-radius: 3px;
  word-break: break-all;
  border: 1px solid var(--border);
}
.val-expected { color: var(--pass); background: var(--pass-bg); border-color: var(--pass-border); }
.val-actual   { color: var(--fail); background: var(--fail-bg); border-color: var(--fail-border); }

/* ── Interaction results table ───────────────────── */
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.results-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}
.results-table th {
  text-align: left;
  padding: 0.55rem 1rem;
  font-size: 0.6875rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-muted);
  background: #fafaf9;
  border-bottom: 1px solid var(--border);
}
.results-table td {
  padding: 0.6rem 1rem;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.results-table tr:last-child td { border-bottom: none; }
.results-table tr:hover td { background: #fafaf9; }
.duration {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

/* ── Status badge ────────────────────────────────── */
.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  padding: 0.2em 0.6em;
  border-radius: 3px;
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  white-space: nowrap;
}
.badge-pass    { background: var(--pass-bg);    color: var(--pass);    border: 1px solid var(--pass-border); }
.badge-fail    { background: var(--fail-bg);    color: var(--fail);    border: 1px solid var(--fail-border); }
.badge-pending { background: var(--pending-bg); color: var(--pending); border: 1px solid var(--pending-border); }

/* ── Footer ──────────────────────────────────────── */
footer {
  max-width: 1060px;
  margin: 0 auto;
  padding: 1.25rem 1.5rem;
  text-align: center;
  font-size: 0.75rem;
  color: var(--text-muted);
  border-top: 1px solid var(--border);
}
footer a { color: var(--accent); text-decoration: none; }
footer a:hover { text-decoration: underline; }

/* ── Responsive ──────────────────────────────────── */
@media (max-width: 640px) {
  .stat-grid { grid-template-columns: repeat(2, 1fr); }
  .stat-card + .stat-card { border-left: none; border-top: 1px solid var(--border); }
  .stat-grid .stat-card:nth-child(2n+1) { border-left: none; }
  .stat-grid .stat-card:nth-child(2n)   { border-left: 1px solid var(--border); }
  .mismatch-table { font-size: 0.75rem; }
  .col-location { width: auto; }
  .report-title { font-size: 1.5rem; }
  .page-wrap { padding: 0 1rem 2rem; }
}

/* ── Print ───────────────────────────────────────── */
@media print {
  * { -webkit-print-color-adjust: exact; print-color-adjust: exact; }

  body { background: #fff; font-size: 11pt; }

  .report-header { box-shadow: none; border-top-width: 4pt; margin-bottom: 1.25rem; }
  .page-wrap { padding: 0; max-width: none; gap: 1.25rem; }

  section h2 { font-size: 7pt; }

  .stat-grid { border: 1pt solid var(--border); }
  .stat-card { padding: 1rem 1rem 1.25rem; }
  .stat-value { font-size: 2rem; }

  .error-card { break-inside: avoid; page-break-inside: avoid; margin-bottom: 0.5rem; }
  .error-card-body { padding: 0.75rem; }

  section { break-inside: avoid; page-break-inside: avoid; }
  .card { break-inside: avoid; page-break-inside: avoid; }
  .results-table tr { break-inside: avoid; page-break-inside: avoid; }

  .mismatch-table tr:hover td,
  .results-table tr:hover td { background: inherit; }

  footer { margin-top: 1.5rem; }
}
        </style>
      </head>
      <body>

        <!-- ── Header ─────────────────────────────────── -->
        <div class="report-header">
          <div class="report-header-inner">
            <div>
              <div class="report-meta">Pact &#183; Verification Report</div>
              <h1 class="report-title"><xsl:value-of select="provider"/></h1>
              <p class="report-subtitle">Provider Contract Verification</p>
            </div>
            <xsl:choose>
              <xsl:when test="result = 'true'">
                <div class="status-stamp pass">
                  <div class="stamp-icon">&#10003;</div>
                  <div class="stamp-text">Passed</div>
                </div>
              </xsl:when>
              <xsl:otherwise>
                <div class="status-stamp fail">
                  <div class="stamp-icon">&#10007;</div>
                  <div class="stamp-text">Failed</div>
                </div>
              </xsl:otherwise>
            </xsl:choose>
          </div>
        </div>

        <div class="page-wrap">

          <!-- ── Summary cards ──────────────────────── -->
          <section>
            <h2>Summary</h2>
            <div class="stat-grid">
              <div class="stat-card">
                <div class="stat-value">
                  <xsl:value-of select="count(interaction_results/interaction)"/>
                </div>
                <div class="stat-label">Total</div>
              </div>
              <div class="stat-card pass">
                <div class="stat-value">
                  <xsl:value-of select="count(interaction_results/interaction[result = 'OK'])"/>
                </div>
                <div class="stat-label">Passed</div>
              </div>
              <div class="stat-card fail">
                <div class="stat-value">
                  <xsl:value-of select="count(errors/error)"/>
                </div>
                <div class="stat-label">Failed</div>
              </div>
              <div class="stat-card pending">
                <div class="stat-value">
                  <xsl:value-of select="count(pending_errors/error)"/>
                </div>
                <div class="stat-label">Pending</div>
              </div>
            </div>
          </section>

          <!-- ── Notices ────────────────────────────── -->
          <xsl:if test="notices/notice">
            <section>
              <h2>Notices</h2>
              <xsl:for-each select="notices/notice">
                <div class="notice">
                  <span class="notice-icon">&#8505;</span>
                  <span><xsl:value-of select="entry[key = 'text']/value"/></span>
                </div>
              </xsl:for-each>
            </section>
          </xsl:if>

          <!-- ── Failures ───────────────────────────── -->
          <xsl:if test="errors/error">
            <section>
              <h2>Failures</h2>
              <xsl:for-each select="errors/error">
                <xsl:call-template name="error-card">
                  <xsl:with-param name="style">fail</xsl:with-param>
                </xsl:call-template>
              </xsl:for-each>
            </section>
          </xsl:if>

          <!-- ── Pending Failures ───────────────────── -->
          <xsl:if test="pending_errors/error">
            <section>
              <h2>Pending Failures</h2>
              <xsl:for-each select="pending_errors/error">
                <xsl:call-template name="error-card">
                  <xsl:with-param name="style">pending</xsl:with-param>
                </xsl:call-template>
              </xsl:for-each>
            </section>
          </xsl:if>

          <!-- ── Interaction Results ────────────────── -->
          <xsl:if test="interaction_results/interaction">
            <section>
              <h2>Interaction Results</h2>
              <div class="card">
                <table class="results-table">
                  <thead>
                    <tr>
                      <th>Interaction</th>
                      <th>Status</th>
                      <th>Duration</th>
                    </tr>
                  </thead>
                  <tbody>
                    <xsl:for-each select="interaction_results/interaction">
                      <tr>
                        <td><xsl:value-of select="description"/></td>
                        <td>
                          <xsl:choose>
                            <xsl:when test="pending = 'true'">
                              <span class="badge badge-pending">Pending</span>
                            </xsl:when>
                            <xsl:when test="result = 'OK'">
                              <span class="badge badge-pass">&#10003; Passed</span>
                            </xsl:when>
                            <xsl:otherwise>
                              <span class="badge badge-fail">&#10007; Failed</span>
                            </xsl:otherwise>
                          </xsl:choose>
                        </td>
                        <td class="duration">
                          <xsl:value-of select="duration_ms"/>
                          <xsl:text> ms</xsl:text>
                        </td>
                      </tr>
                    </xsl:for-each>
                  </tbody>
                </table>
              </div>
            </section>
          </xsl:if>

        </div>

        <footer>
          Generated by <a href="https://pact.io">Pact</a> &#183; pact_verifier_cli
        </footer>

      </body>
    </html>
  </xsl:template>

  <!-- ============================================================
       ERROR CARD — renders one <error> element from errors or
       pending_errors.  Call with the current node set to <error>.
       $style: "fail" | "pending"
       ============================================================ -->
  <xsl:template name="error-card">
    <xsl:param name="style">fail</xsl:param>
    <div class="error-card {$style}">
      <div class="error-card-head">
        <span class="error-kind-tag">
          <xsl:choose>
            <xsl:when test="$style = 'pending'">Pending</xsl:when>
            <xsl:otherwise>Failure</xsl:otherwise>
          </xsl:choose>
        </span>
        <span class="error-card-title">
          <xsl:value-of select="interaction"/>
        </span>
      </div>
      <div class="error-card-body">
        <xsl:choose>

          <!-- Simple error string -->
          <xsl:when test="mismatch/@type = 'error'">
            <p class="error-msg">
              <xsl:value-of select="mismatch/error_message"/>
            </p>
          </xsl:when>

          <!-- Structured mismatches table -->
          <xsl:when test="mismatch/@type = 'mismatches'">
            <table class="mismatch-table">
              <thead>
                <tr>
                  <th class="col-type">Type</th>
                  <th class="col-location">Location</th>
                  <th>Description</th>
                  <th>Expected</th>
                  <th>Actual</th>
                </tr>
              </thead>
              <tbody>
                <xsl:for-each select="mismatch/mismatches/mismatch">
                  <tr>
                    <td>
                      <span class="type-chip">
                        <xsl:call-template name="mismatch-label">
                          <xsl:with-param name="type" select="@type"/>
                        </xsl:call-template>
                      </span>
                    </td>
                    <td>
                      <xsl:if test="path">
                        <code><xsl:value-of select="path"/></code>
                      </xsl:if>
                      <xsl:if test="key">
                        <code><xsl:value-of select="key"/></code>
                      </xsl:if>
                      <xsl:if test="parameter">
                        <code><xsl:value-of select="parameter"/></code>
                      </xsl:if>
                    </td>
                    <td><xsl:value-of select="description"/></td>
                    <td>
                      <xsl:if test="expected">
                        <code class="val-expected"><xsl:value-of select="expected"/></code>
                      </xsl:if>
                    </td>
                    <td>
                      <xsl:if test="actual">
                        <code class="val-actual"><xsl:value-of select="actual"/></code>
                      </xsl:if>
                    </td>
                  </tr>
                </xsl:for-each>
              </tbody>
            </table>
          </xsl:when>

        </xsl:choose>
      </div>
    </div>
  </xsl:template>

  <!-- ============================================================
       MISMATCH TYPE LABEL
       Converts the @type attribute value to a short human label.
       ============================================================ -->
  <xsl:template name="mismatch-label">
    <xsl:param name="type"/>
    <xsl:choose>
      <xsl:when test="$type = 'MethodMismatch'">Method</xsl:when>
      <xsl:when test="$type = 'PathMismatch'">Path</xsl:when>
      <xsl:when test="$type = 'StatusMismatch'">Status</xsl:when>
      <xsl:when test="$type = 'QueryMismatch'">Query Param</xsl:when>
      <xsl:when test="$type = 'HeaderMismatch'">Header</xsl:when>
      <xsl:when test="$type = 'BodyTypeMismatch'">Body Type</xsl:when>
      <xsl:when test="$type = 'BodyMismatch'">Body</xsl:when>
      <xsl:when test="$type = 'MetadataMismatch'">Metadata</xsl:when>
      <xsl:otherwise><xsl:value-of select="$type"/></xsl:otherwise>
    </xsl:choose>
  </xsl:template>

</xsl:stylesheet>
