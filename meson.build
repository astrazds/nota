project(
  'noter',
  version: '2.0.0-alpha.1',
  meson_version: '>= 1.3.0',
)

python = find_program('python3')
noter_binary = custom_target(
  'noter-desktop',
  output: 'noter-desktop',
  command: [
    python,
    files('build-aux/cargo-build.py'),
    meson.project_source_root(),
    meson.current_build_dir() / 'cargo-target',
    '@OUTPUT@',
  ],
  build_by_default: true,
  install: true,
  install_dir: get_option('bindir'),
)

install_data(
  'data/net.astrazds.Nota.desktop',
  install_dir: get_option('datadir') / 'applications',
)
install_data(
  'data/net.astrazds.Nota.metainfo.xml',
  install_dir: get_option('datadir') / 'metainfo',
)
install_data(
  'assets/icons/noter-icon.svg',
  rename: 'net.astrazds.Nota.svg',
  install_dir: get_option('datadir') / 'icons/hicolor/scalable/apps',
)
install_data(
  [
    'assets/fonts/source-sans-3-latin-wght-normal.woff2',
    'assets/fonts/source-sans-3-latin-wght-italic.woff2',
    'assets/fonts/source-code-pro-latin-wght-normal.woff2',
    'assets/fonts/source-code-pro-latin-wght-italic.woff2',
  ],
  install_dir: get_option('datadir') / 'net.astrazds.Nota/fonts',
)

test(
  'appimage-appdir-contract',
  python,
  args: [files('build-aux/test_package_appimage.py')],
  workdir: meson.project_source_root() / 'build-aux',
)
