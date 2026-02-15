/**
 * Pure frontend EPUB Table of Contents parser.
 *
 * Parses the TOC from within an EPUB archive by fetching:
 *   META-INF/container.xml → content.opf → toc.ncx (EPUB2) or nav doc (EPUB3)
 *
 * Uses DOMParser for all XML/HTML parsing — no external dependencies.
 */

import type { EpubTocItem } from "@/api/types";

/**
 * Resolves a potentially relative href against a base directory within the EPUB.
 */
function resolveEpubHref(href: string, baseDir: string): string {
    if (!href || href.startsWith("http://") || href.startsWith("https://")) {
        return href;
    }

    const combined = baseDir ? `${baseDir}/${href}` : href;
    const parts = combined.split("/");
    const resolved: string[] = [];

    for (const part of parts) {
        if (part === "..") {
            resolved.pop();
        } else if (part !== "." && part !== "") {
            resolved.push(part);
        }
    }

    return resolved.join("/");
}

/**
 * Extracts the base directory from a path (everything before the last `/`).
 */
function baseDirOf(path: string): string {
    const lastSlash = path.lastIndexOf("/");
    return lastSlash >= 0 ? path.substring(0, lastSlash) : "";
}

/**
 * Parses an EPUB2 toc.ncx document into a tree of EpubTocItem.
 */
function parseNcxToc(ncxXml: string, baseDir: string): EpubTocItem[] {
    const parser = new DOMParser();
    const doc = parser.parseFromString(ncxXml, "application/xml");

    const parseNavPoints = (parent: Element): EpubTocItem[] => {
        const items: EpubTocItem[] = [];
        // Only direct child navPoints (not nested ones from children)
        for (const navPoint of parent.children) {
            if (navPoint.localName !== "navPoint") continue;

            const title =
                navPoint
                    .querySelector("navLabel > text")
                    ?.textContent?.trim() || "";

            const rawSrc =
                navPoint.querySelector("content")?.getAttribute("src") || "";

            const src = rawSrc ? resolveEpubHref(rawSrc, baseDir) : "";

            const playOrderAttr = navPoint.getAttribute("playOrder");
            const playOrder = playOrderAttr
                ? parseInt(playOrderAttr, 10)
                : undefined;

            const children = parseNavPoints(navPoint);

            items.push({ title, src, children, playOrder });
        }

        return items;
    };

    // The navMap is the root of the TOC tree in NCX
    const navMap = doc.querySelector("navMap");
    if (!navMap) return [];

    return parseNavPoints(navMap);
}

/**
 * Parses an EPUB3 navigation document (XHTML with <nav epub:type="toc">).
 */
function parseNavToc(navHtml: string, baseDir: string): EpubTocItem[] {
    const parser = new DOMParser();
    const doc = parser.parseFromString(navHtml, "application/xhtml+xml");

    // Find <nav epub:type="toc"> — try namespace-aware first, then fallback
    let navElement: Element | null = null;

    for (const nav of doc.querySelectorAll("nav")) {
        const epubType =
            nav.getAttributeNS("http://www.idpf.org/2007/ops", "type") ||
            nav.getAttribute("epub:type") ||
            "";
        if (epubType === "toc") {
            navElement = nav;
            break;
        }
    }

    if (!navElement) return [];

    const parseOlItems = (ol: Element): EpubTocItem[] => {
        const items: EpubTocItem[] = [];

        for (const li of ol.children) {
            if (li.localName !== "li") continue;

            const anchor = li.querySelector(":scope > a");
            const span = li.querySelector(":scope > span");

            const title =
                anchor?.textContent?.trim() || span?.textContent?.trim() || "";

            const rawHref = anchor?.getAttribute("href") || "";
            const src = rawHref ? resolveEpubHref(rawHref, baseDir) : "";

            const nestedOl = li.querySelector(":scope > ol");
            const children = nestedOl ? parseOlItems(nestedOl) : [];

            items.push({ title, src, children });
        }

        return items;
    };

    const rootOl = navElement.querySelector(":scope > ol");
    if (!rootOl) return [];

    return parseOlItems(rootOl);
}

/**
 * Main entry point: parses the EPUB TOC from within an EPUB archive.
 *
 * @param fetchResource - A function that fetches a resource from within the EPUB
 *                        archive by its internal ZIP path and returns the text content.
 * @returns Parsed table of contents as EpubTocItem[]
 */
export async function parseEpubToc(
    fetchResource: (path: string) => Promise<string>,
): Promise<EpubTocItem[]> {
    // 1. Parse container.xml to find the OPF path
    let containerXml: string;
    try {
        containerXml = await fetchResource("META-INF/container.xml");
    } catch {
        console.warn("EPUB TOC: Failed to fetch container.xml");
        return [];
    }

    const containerParser = new DOMParser();
    const containerDoc = containerParser.parseFromString(
        containerXml,
        "application/xml",
    );
    const rootfileEl = containerDoc.querySelector("rootfile[full-path]");
    const opfPath = rootfileEl?.getAttribute("full-path");
    if (!opfPath) {
        console.warn("EPUB TOC: No rootfile found in container.xml");
        return [];
    }

    // 2. Parse the OPF to find the TOC reference
    let opfXml: string;
    try {
        opfXml = await fetchResource(opfPath);
    } catch {
        console.warn("EPUB TOC: Failed to fetch OPF:", opfPath);
        return [];
    }

    const opfParser = new DOMParser();
    const opfDoc = opfParser.parseFromString(opfXml, "application/xml");
    const opfBaseDir = baseDirOf(opfPath);

    // Try EPUB3 nav document first: look for <item properties="nav">
    const manifest = opfDoc.querySelector("manifest");
    let navItem: Element | null = null;
    let ncxItem: Element | null = null;

    if (manifest) {
        for (const item of manifest.querySelectorAll("item")) {
            const props = item.getAttribute("properties") || "";
            if (props.split(/\s+/).includes("nav")) {
                navItem = item;
            }
            const mediaType = item.getAttribute("media-type") || "";
            if (mediaType === "application/x-dtbncx+xml") {
                ncxItem = item;
            }
        }
    }

    // Also check for NCX via <spine toc="..."> idref
    if (!ncxItem) {
        const spine = opfDoc.querySelector("spine");
        const tocId = spine?.getAttribute("toc");
        if (tocId && manifest) {
            ncxItem = manifest.querySelector(`item[id="${tocId}"]`);
        }
    }

    // Try EPUB3 nav document first
    if (navItem) {
        const navHref = navItem.getAttribute("href");
        if (navHref) {
            const navPath = resolveEpubHref(navHref, opfBaseDir);
            try {
                const navHtml = await fetchResource(navPath);
                const navBaseDir = baseDirOf(navPath);
                const tocItems = parseNavToc(navHtml, navBaseDir);
                if (tocItems.length > 0) {
                    return tocItems;
                }
            } catch {
                console.warn(
                    "EPUB TOC: Failed to parse EPUB3 nav document, trying NCX fallback",
                );
            }
        }
    }

    // Fall back to NCX
    if (ncxItem) {
        const ncxHref = ncxItem.getAttribute("href");
        if (ncxHref) {
            const ncxPath = resolveEpubHref(ncxHref, opfBaseDir);
            try {
                const ncxXml = await fetchResource(ncxPath);
                const ncxBaseDir = baseDirOf(ncxPath);
                return parseNcxToc(ncxXml, ncxBaseDir);
            } catch {
                console.warn("EPUB TOC: Failed to parse NCX TOC");
            }
        }
    }

    console.warn("EPUB TOC: No TOC found in EPUB");
    return [];
}
