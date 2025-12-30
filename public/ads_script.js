window.triggerAdSense = function () {
  try {
    // Just tell AdSense to scan for a new slot.
    // It will find the <ins> tag we just added because it lacks the 'data-adsbygoogle-status' attribute.
    (adsbygoogle = window.adsbygoogle || []).push({});
  } catch (e) {
    console.error("AdSense trigger failed:", e);
  }
};
