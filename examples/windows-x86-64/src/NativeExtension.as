package src {
    import flash.external.ExtensionContext;
    import flash.external.ExtensionInfo;
    import flash.events.StatusEvent;
    public final class NativeExtension {
        public static const ID:String = "native-extension-id";
        private var _ctx:ExtensionContext;
        private var _info:ExtensionInfo;
        function NativeExtension() {
            _ctx = ExtensionContext.createExtensionContext(ID, "test");
            _info = ExtensionContext.getExtensionInfo(ID);
            if (!_ctx) {
                if (_info) {
                    throw new Error("Extension initialization failed.");
                } else {
                    throw new Error("Extension is not loaded.");
                }
            }
            _ctx.addEventListener(StatusEvent.STATUS, function(evt:StatusEvent):void {
                if (evt.level=="debug" || evt.level=="warning" || evt.level=="error") {
                    trace("Extension `"+ID+"`: "+evt)
                }
            }, false, int.MAX_VALUE);
            trace("Extension: "+ID+", Methods: "+_ctx.functions);
        }
        public function hello(words: String):String {
            return _ctx.call("hello", words) as String;
        }
    }
}