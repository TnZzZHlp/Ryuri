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
