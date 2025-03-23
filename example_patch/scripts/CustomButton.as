package com.ui.components {
    import flash.display.MovieClip;
    import flash.events.MouseEvent;

    public class ButtonBase extends MovieClip {
        protected var isEnabled:Boolean = true;
        protected var isHovered:Boolean = false;
        protected var isPressed:Boolean = false;

        public function ButtonBase() {
            super();
            initialize();
        }

        protected function initialize():void {
            // Set up event listeners
            addEventListener(MouseEvent.MOUSE_OVER, onMouseOver);
            addEventListener(MouseEvent.MOUSE_OUT, onMouseOut);
            addEventListener(MouseEvent.MOUSE_DOWN, onMouseDown);
            addEventListener(MouseEvent.MOUSE_UP, onMouseUp);

            // Initial state
            updateVisualState();
        }

        protected function onMouseOver(event:MouseEvent):void {
            if (!isEnabled) return;
            isHovered = true;
            updateVisualState();
        }

        protected function onMouseOut(event:MouseEvent):void {
            if (!isEnabled) return;
            isHovered = false;
            isPressed = false;
            updateVisualState();
        }

        protected function onMouseDown(event:MouseEvent):void {
            if (!isEnabled) return;
            isPressed = true;
            updateVisualState();
        }

        protected function onMouseUp(event:MouseEvent):void {
            if (!isEnabled) return;
            isPressed = false;
            updateVisualState();
        }

        protected function updateVisualState():void {
            // Update alpha based on state
            if (!isEnabled) {
                alpha = 0.5;
            } else if (isPressed) {
                alpha = 0.8;
            } else if (isHovered) {
                alpha = 1.0;
            } else {
                alpha = 0.9;
            }
        }

        public function set enabled(value:Boolean):void {
            isEnabled = value;
            updateVisualState();
            mouseEnabled = value;
        }

        public function get enabled():Boolean {
            return isEnabled;
        }
    }
}
