async function builtCss(page) {
  return page.evaluate(async () => {
    const hrefs = Array.from(document.querySelectorAll("link[rel='stylesheet']"), (link) => link.href);
    const styles = await Promise.all(hrefs.map((href) => fetch(href).then((response) => response.text())));
    return styles.join("\n");
  });
}

module.exports = {
  builtCss,
};
