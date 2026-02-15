import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function isSkippableResourceUrl(url: string): boolean {
  return (
    !url ||
    url.startsWith("data:") ||
    url.startsWith("blob:") ||
    url.startsWith("javascript:") ||
    url.startsWith("#")
  )
}

export function resolveRelativeUrlWithBaseQuery(
  rawUrl: string,
  baseUrl: string
): string | null {
  try {
    const origin =
      typeof window !== "undefined" ? window.location.origin : "http://localhost"
    const base = new URL(baseUrl, origin)
    const resolved = new URL(rawUrl, base)
    base.searchParams.forEach((value, key) => {
      if (!resolved.searchParams.has(key)) {
        resolved.searchParams.set(key, value)
      }
    })
    return resolved.toString()
  } catch {
    return null
  }
}

export function rewriteCssUrlsWithBase(
  cssText: string,
  stylesheetHref: string
): string {
  let rewritten = cssText.replace(
    /url\(\s*(["']?)(.*?)\1\s*\)/gi,
    (_match, quote: string, url: string) => {
      const candidate = url.trim()
      if (isSkippableResourceUrl(candidate)) {
        return `url(${quote}${candidate}${quote})`
      }
      const resolved = resolveRelativeUrlWithBaseQuery(candidate, stylesheetHref)
      return `url(${quote}${resolved ?? candidate}${quote})`
    }
  )

  rewritten = rewritten.replace(
    /@import\s+url\(\s*(["']?)(.*?)\1\s*\)/gi,
    (_match, quote: string, url: string) => {
      const candidate = url.trim()
      if (isSkippableResourceUrl(candidate)) {
        return `@import url(${quote}${candidate}${quote})`
      }
      const resolved = resolveRelativeUrlWithBaseQuery(candidate, stylesheetHref)
      return `@import url(${quote}${resolved ?? candidate}${quote})`
    }
  )

  rewritten = rewritten.replace(
    /@import\s+(["'])(.*?)\1/gi,
    (_match, quote: string, url: string) => {
      const candidate = url.trim()
      if (isSkippableResourceUrl(candidate)) {
        return `@import ${quote}${candidate}${quote}`
      }
      const resolved = resolveRelativeUrlWithBaseQuery(candidate, stylesheetHref)
      return `@import ${quote}${resolved ?? candidate}${quote}`
    }
  )

  return rewritten
}

export function splitEpubSrc(src: string): { path: string; fragment?: string } {
  const [pathPart, fragmentPart] = src.split("#", 2)
  const path = normalizeEpubPath(pathPart ?? "")
  const fragment = fragmentPart?.trim() ? fragmentPart.trim() : undefined
  return { path, fragment }
}

export function normalizeEpubPath(path: string): string {
  const raw = (path ?? "").trim().replace(/\\/g, "/")
  if (!raw) return ""
  if (raw.includes("://")) return raw

  const queryIndex = raw.indexOf("?")
  const pathPart = queryIndex >= 0 ? raw.slice(0, queryIndex) : raw
  const queryPart = queryIndex >= 0 ? raw.slice(queryIndex + 1) : undefined
  const segments: string[] = []
  for (const segment of pathPart.split("/")) {
    if (!segment || segment === ".") continue
    if (segment === "..") {
      segments.pop()
      continue
    }
    segments.push(segment)
  }

  const normalized = segments.join("/")
  if (!queryPart) return normalized
  return `${normalized}?${queryPart}`
}

export function isLooseEpubPathMatch(left: string, right: string): boolean {
  const leftNormalized = normalizeEpubPath(left)
  const rightNormalized = normalizeEpubPath(right)
  if (!leftNormalized || !rightNormalized) return false
  if (leftNormalized === rightNormalized) return true

  return (
    leftNormalized.endsWith(`/${rightNormalized}`) ||
    rightNormalized.endsWith(`/${leftNormalized}`)
  )
}
