require 'json'

package = JSON.parse(File.read(File.join(__dir__, 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'ExpoOriha'
  s.version        = package['version']
  s.summary        = package['description']
  s.description    = package['description']
  s.license        = 'MIT OR Apache-2.0'
  s.author         = 'Progressive Works, Inc.'
  s.homepage       = 'https://github.com/craftguild/oriha.rs'
  s.platforms      = { :ios => '15.1' }
  s.source         = { :path => '.' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'
  s.source_files = ['ios/*.{h,m,mm,swift}', 'ios/generated/*.{h,swift}']
  s.public_header_files = 'ios/generated/*.h'
  s.vendored_frameworks = 'ios/native/OrihaNativeBridge.xcframework'
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'HEADER_SEARCH_PATHS' => '"$(PODS_TARGET_SRCROOT)/ios/generated"',
    'SWIFT_INCLUDE_PATHS' => '"$(PODS_TARGET_SRCROOT)/ios/generated"'
  }
end
