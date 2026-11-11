// visionOS gives a window no screen: `UIScreen` and `UIWindow.screen` are unavailable there. The iOS
// presenter reads a window's screen for its scale, refresh ceiling and coordinate spaces, so on visionOS
// this stand-in, which shadows UIKit's names in Exact's modules, answers each from the window itself.
#if os(visionOS)
import UIKit

package final class ExactVisionScreen {
    private weak var window: UIWindow?
    init(_ window: UIWindow?) { self.window = window }

    package static var main: ExactVisionScreen {
        ExactVisionScreen(UIApplication.shared.connectedScenes.compactMap { ($0 as? UIWindowScene)?.keyWindow }.first)
    }
    package var scale: CGFloat { window?.traitCollection.displayScale ?? UITraitCollection.current.displayScale }
    /// Apple Vision Pro draws at 90 Hz unless an app asks for more.
    package var maximumFramesPerSecond: Int { 90 }
    package var bounds: CGRect { window?.bounds ?? .zero }
    package var coordinateSpace: UICoordinateSpace { window ?? UIView() }
    package var fixedCoordinateSpace: UICoordinateSpace { coordinateSpace }
    package var traitCollection: UITraitCollection { window?.traitCollection ?? .current }
    /// visionOS reports no extended dynamic range headroom for a window.
    package var potentialEDRHeadroom: CGFloat { 1 }
}

package typealias UIScreen = ExactVisionScreen

extension UIWindow {
    package var screen: UIScreen { UIScreen(self) }
}

extension UIWindowScene {
    package var screen: UIScreen { UIScreen(keyWindow) }
}

// visionOS has no glass container effect either. A glass group refuses with
// "os" there (`glassGroupConflict`), so this stand-in only lets the shared
// code compile: it is never shown and draws nothing.
final class UIGlassContainerEffect: UIVisualEffect {
    var spacing: CGFloat = 0
}
#endif
