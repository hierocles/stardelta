package com.ui.features {
    import flash.display.MovieClip;
    import flash.events.Event;
    import com.ui.components.ButtonBase;

    public class NewFeature extends MovieClip {
        private var buttons:Vector.<ButtonBase>;
        private var isInitialized:Boolean = false;

        public function NewFeature() {
            super();
            buttons = new Vector.<ButtonBase>();

            if (stage) {
                init();
            } else {
                addEventListener(Event.ADDED_TO_STAGE, onAddedToStage);
            }
        }

        private function onAddedToStage(event:Event):void {
            removeEventListener(Event.ADDED_TO_STAGE, onAddedToStage);
            init();
        }

        private function init():void {
            if (isInitialized) return;
            isInitialized = true;

            // Find all ButtonBase instances on stage
            findButtons(this);

            // Add feature-specific event listeners
            stage.addEventListener(Event.RESIZE, onStageResize);

            // Initial layout
            updateLayout();
        }

        private function findButtons(container:MovieClip):void {
            for (var i:int = 0; i < container.numChildren; i++) {
                var child:* = container.getChildAt(i);
                if (child is ButtonBase) {
                    buttons.push(child as ButtonBase);
                } else if (child is MovieClip) {
                    findButtons(child as MovieClip);
                }
            }
        }

        private function onStageResize(event:Event):void {
            updateLayout();
        }

        private function updateLayout():void {
            var padding:Number = 10;
            var currentY:Number = padding;

            for each (var button:ButtonBase in buttons) {
                button.x = padding;
                button.y = currentY;
                currentY += button.height + padding;
            }
        }

        public function cleanup():void {
            if (stage) {
                stage.removeEventListener(Event.RESIZE, onStageResize);
            }
            buttons.length = 0;
            isInitialized = false;
        }
    }
}
