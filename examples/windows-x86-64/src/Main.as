package src {
    import flash.display.Sprite;
    import src.NativeExtension;
    public final class Main extends Sprite {
        private const EX:NativeExtension = new NativeExtension();
        function Main(){
            trace(EX.hello("Hello! Extension."));
        }
    }
}