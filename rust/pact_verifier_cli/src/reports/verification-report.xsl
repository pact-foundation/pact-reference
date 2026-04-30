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
:root {
  --pass:        #166534;
  --pass-bg:     #dcfce7;
  --fail:        #991b1b;
  --fail-bg:     #fee2e2;
  --pending:     #92400e;
  --pending-bg:  #fef3c7;
  --info:        #1e40af;
  --info-bg:     #dbeafe;
  --header:      #1e1b4b;
  --text:        #111827;
  --muted:       #6b7280;
  --border:      #e5e7eb;
  --surface:     #ffffff;
  --bg:          #f3f4f6;
  --radius:      8px;
  --shadow:      0 1px 3px rgba(0,0,0,.12), 0 1px 2px rgba(0,0,0,.08);
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.55;
  min-height: 100vh;
}

/* ── Header ─────────────────────────────────────── */
.site-header {
  background: var(--header);
  color: #fff;
  padding: 1.25rem 2rem 1rem;
}
.header-inner {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: .75rem;
}
.header-left {
  display: flex;
  align-items: center;
  gap: .85rem;
}
.pact-badge {
  background: #e11d48;
  color: #fff;
  font-size: .8rem;
  font-weight: 800;
  letter-spacing: .2em;
  padding: .2em .6em;
  border-radius: 4px;
  flex-shrink: 0;
}
.header-left h1 { font-size: 1.2rem; font-weight: 600; }
.provider-name {
  max-width: 1100px;
  margin: .55rem auto 0;
  font-size: .875rem;
  opacity: .7;
}

/* ── Badges ──────────────────────────────────────── */
.badge {
  display: inline-block;
  padding: .3em .85em;
  border-radius: 999px;
  font-weight: 700;
  font-size: .8rem;
  letter-spacing: .04em;
  white-space: nowrap;
}
.badge-lg    { font-size: .95rem; padding: .35em 1.1em; }
.badge-pass    { background: var(--pass-bg);    color: var(--pass);    }
.badge-fail    { background: var(--fail-bg);    color: var(--fail);    }
.badge-pending { background: var(--pending-bg); color: var(--pending); }

/* ── Page layout ─────────────────────────────────── */
main {
  max-width: 1100px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}
section h2 {
  font-size: .72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: .1em;
  color: var(--muted);
  margin-bottom: 1rem;
}

/* ── Summary stat cards ──────────────────────────── */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
}
.stat-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.25rem 1.5rem;
  box-shadow: var(--shadow);
  text-align: center;
}
.stat-value {
  font-size: 2.25rem;
  font-weight: 700;
  line-height: 1;
  margin-bottom: .3rem;
}
.stat-label {
  font-size: .72rem;
  text-transform: uppercase;
  letter-spacing: .09em;
  color: var(--muted);
}
.stat-card.pass    .stat-value { color: var(--pass);    }
.stat-card.fail    .stat-value { color: var(--fail);    }
.stat-card.pending .stat-value { color: var(--pending); }

/* ── Notice cards ────────────────────────────────── */
.notice {
  background: var(--info-bg);
  border-left: 4px solid #3b82f6;
  border-radius: var(--radius);
  padding: .75rem 1rem;
  color: var(--info);
  font-size: .9rem;
}
.notice + .notice { margin-top: .5rem; }

/* ── Error / Pending cards ───────────────────────── */
.error-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  overflow: hidden;
  margin-bottom: 1rem;
}
.error-card:last-child { margin-bottom: 0; }

.error-card-head {
  display: flex;
  align-items: center;
  gap: .6rem;
  padding: .8rem 1.2rem;
  border-bottom: 1px solid var(--border);
  background: #fafafa;
}
.error-card.fail    .error-card-head { border-left: 4px solid var(--fail);  }
.error-card.pending .error-card-head { border-left: 4px solid #f59e0b;       }

.error-card-title {
  font-weight: 600;
  font-size: .93rem;
  word-break: break-all;
}
.error-card-body { padding: 1rem 1.2rem; }

/* ── Error message (type=error) ──────────────────── */
.error-msg {
  font-family: ui-monospace, 'Cascadia Code', 'Fira Code', Consolas, monospace;
  font-size: .84rem;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 4px;
  padding: .65rem .9rem;
  color: #991b1b;
  word-break: break-all;
}

/* ── Mismatch detail table ───────────────────────── */
.mismatch-table {
  width: 100%;
  border-collapse: collapse;
  font-size: .86rem;
}
.mismatch-table th {
  text-align: left;
  padding: .5rem .8rem;
  font-size: .7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: .07em;
  color: var(--muted);
  border-bottom: 2px solid var(--border);
  white-space: nowrap;
}
.mismatch-table td {
  padding: .55rem .8rem;
  vertical-align: top;
  border-bottom: 1px solid var(--border);
}
.mismatch-table tr:last-child td { border-bottom: none; }
.mismatch-table tr:hover td { background: #fafafa; }
.col-type     { width: 8rem; }
.col-location { width: 11rem; }

/* ── Type chip ───────────────────────────────────── */
.type-chip {
  display: inline-block;
  font-size: .67rem;
  font-weight: 700;
  letter-spacing: .05em;
  text-transform: uppercase;
  padding: .15em .55em;
  border-radius: 4px;
  white-space: nowrap;
}
.error-card.fail    .type-chip { background: var(--fail-bg);    color: var(--fail);    }
.error-card.pending .type-chip { background: var(--pending-bg); color: var(--pending); }

/* ── Inline code ─────────────────────────────────── */
code {
  font-family: ui-monospace, 'Cascadia Code', 'Fira Code', Consolas, monospace;
  font-size: .82em;
  background: #f3f4f6;
  padding: .1em .35em;
  border-radius: 3px;
  word-break: break-all;
}
.val-expected { color: var(--pass); }
.val-actual   { color: var(--fail); }

/* ── Interaction results table ───────────────────── */
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  overflow: hidden;
}
.results-table {
  width: 100%;
  border-collapse: collapse;
  font-size: .875rem;
}
.results-table th {
  text-align: left;
  padding: .65rem 1rem;
  font-size: .7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: .07em;
  color: var(--muted);
  border-bottom: 2px solid var(--border);
  background: #fafafa;
}
.results-table td {
  padding: .65rem 1rem;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.results-table tr:last-child td { border-bottom: none; }
.results-table tr:hover td { background: #fafafa; }
.duration {
  color: var(--muted);
  font-size: .8rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

/* ── Footer ──────────────────────────────────────── */
footer {
  text-align: center;
  padding: 1.5rem;
  font-size: .8rem;
  color: var(--muted);
  border-top: 1px solid var(--border);
  margin-top: auto;
}
footer a { color: #e11d48; text-decoration: none; }
footer a:hover { text-decoration: underline; }

/* ── Responsive ──────────────────────────────────── */
@media (max-width: 640px) {
  .stat-grid { grid-template-columns: repeat(2, 1fr); }
  main { padding: 1.25rem 1rem; }
  .site-header { padding: 1rem; }
  .mismatch-table { font-size: .8rem; }
  .col-location { width: auto; }
}
        </style>
      </head>
      <body>

        <!-- ── Header ─────────────────────────────────── -->
        <header class="site-header">
          <div class="header-inner">
            <div class="header-left">
              <span class="pact-badge">PACT</span>
              <h1>Verification Report</h1>
            </div>
            <xsl:choose>
              <xsl:when test="result = 'true'">
                <span class="badge badge-pass badge-lg">&#10003; PASSED</span>
              </xsl:when>
              <xsl:otherwise>
                <span class="badge badge-fail badge-lg">&#10007; FAILED</span>
              </xsl:otherwise>
            </xsl:choose>
          </div>
          <p class="provider-name">Provider: <xsl:value-of select="provider"/></p>
        </header>

        <main>

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
                  <xsl:value-of select="entry[key = 'text']/value"/>
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
                              <span class="badge badge-pass">Passed</span>
                            </xsl:when>
                            <xsl:otherwise>
                              <span class="badge badge-fail">Failed</span>
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

        </main>

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
