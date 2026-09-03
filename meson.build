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
  'data/net.astrazds.Noter.desktop',
  install_dir: get_option('datadir') / 'applications',
)
install_data(
  'data/net.astrazds.Noter.metainfo.xml',
  install_dir: get_option('datadir') / 'metainfo',
)
install_data(
  'assets/icons/noter-icon.svg',
  rename: 'net.astrazds.Noter.svg',
  install_dir: get_option('datadir') / 'icons/hicolor/scalable/apps',
)
