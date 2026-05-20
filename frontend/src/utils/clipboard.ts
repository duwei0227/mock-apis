/**
 * Copy text to clipboard.
 * Uses the modern Clipboard API when available (HTTPS / localhost),
 * falls back to execCommand for plain HTTP (e.g. LAN IP access on Windows).
 */
export function copyText(text: string): Promise<void> {
  if (navigator.clipboard && window.isSecureContext) {
    return navigator.clipboard.writeText(text)
  }
  return new Promise((resolve, reject) => {
    const el = document.createElement('textarea')
    el.value = text
    el.style.position = 'fixed'
    el.style.top = '0'
    el.style.left = '0'
    el.style.opacity = '0'
    document.body.appendChild(el)
    el.focus()
    el.select()
    const ok = document.execCommand('copy')
    document.body.removeChild(el)
    ok ? resolve() : reject(new Error('copy failed'))
  })
}
