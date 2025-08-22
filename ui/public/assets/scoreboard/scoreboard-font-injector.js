// Ensures embedded SVG documents (loaded via <object>) can use app-provided fonts.
// It injects @font-face rules from /assets/fonts/fonts.css into the SVG's <defs> as a <style> tag.

(function() {
  function fetchFontsCss() {
    return fetch('/assets/fonts/fonts.css', { cache: 'force-cache' })
      .then(function(res) { return res.text(); })
      .catch(function() { return ''; });
  }

  function injectCssIntoSvg(svgDocument, cssText) {
    if (!svgDocument || !cssText) return;
    var svgRoot = svgDocument.documentElement;
    if (!svgRoot || svgRoot.nodeName.toLowerCase() !== 'svg') return;

    // Create or find <defs>
    var defs = svgRoot.querySelector('defs');
    if (!defs) {
      defs = svgDocument.createElementNS('http://www.w3.org/2000/svg', 'defs');
      svgRoot.insertBefore(defs, svgRoot.firstChild);
    }

    // Avoid duplicate injection
    if (defs.querySelector('style[data-font-injected="true"]')) return;

    var style = svgDocument.createElementNS('http://www.w3.org/2000/svg', 'style');
    style.setAttribute('type', 'text/css');
    style.setAttribute('data-font-injected', 'true');
    style.appendChild(svgDocument.createTextNode(cssText));
    defs.appendChild(style);
  }

  function onObjectLoad(objectEl) {
    var svgDoc = objectEl.contentDocument;
    if (!svgDoc) return;

    fetchFontsCss().then(function(cssText) {
      injectCssIntoSvg(svgDoc, cssText);
    });
  }

  function initForObject(selector) {
    var obj = document.querySelector(selector);
    if (!obj) return;
    if (obj.contentDocument) {
      onObjectLoad(obj);
    }
    obj.addEventListener('load', function() { onObjectLoad(obj); });
  }

  // Expose minimal API if needed by other scripts
  window.ScoreboardFontInjector = {
    initForObject: initForObject
  };
})();


