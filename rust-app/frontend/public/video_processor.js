export class VideoProcessor {
    constructor() {
        this.mpSelfieSegmentation = new SelfieSegmentation({locateFile: (file) => {
            return `https://cdn.jsdelivr.net/npm/@mediapipe/selfie_segmentation/${file}`;
        }});

        this.mpSelfieSegmentation.setOptions({
            modelSelection: 1, // 0: landscape, 1: general (better for square/vertical)
        });

        this.mpSelfieSegmentation.onResults(this.onResults.bind(this));
        
        this.canvas = document.createElement('canvas');
        this.ctx = this.canvas.getContext('2d');
        this.active = false;
        this.mode = 'none'; // none, blur, image
        this.backgroundImage = null;

        // Internal processing loop
        this.videoElement = null;
        this.animationFrameId = null;
    }

    async start(stream) {
        try {
            this.videoElement = document.createElement('video');
            this.videoElement.srcObject = stream;
            this.videoElement.playsInline = true;
            this.videoElement.muted = true;
            await this.videoElement.play();
            
            this.active = true;
            this.canvas.width = this.videoElement.videoWidth || 640;
            this.canvas.height = this.videoElement.videoHeight || 480;
            
            await this.mpSelfieSegmentation.initialize();
            this.process();
            return true;
        } catch (e) {
            console.error("Failed to start video processor", e);
            throw e;
        }
    }

    stop() {
        this.active = false;
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
        }
    }

    setMode(mode, imageUrl) {
        this.mode = mode;
        if (mode === 'image' && imageUrl) {
            const img = new Image();
            img.src = imageUrl;
            img.onload = () => {
                this.backgroundImage = img;
            };
        } else {
            this.backgroundImage = null;
        }
    }

    async process() {
        if (!this.active || !this.videoElement) return;

        if (this.mode === 'none') {
            // Pass through directly
            this.ctx.drawImage(this.videoElement, 0, 0, this.canvas.width, this.canvas.height);
        } else {
            await this.mpSelfieSegmentation.send({image: this.videoElement});
        }
        
        if (this.active) {
            this.animationFrameId = requestAnimationFrame(this.process.bind(this));
        }
    }

    onResults(results) {
        this.ctx.save();
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.drawImage(results.segmentationMask, 0, 0, this.canvas.width, this.canvas.height);

        // Only overwrite existing pixels.
        this.ctx.globalCompositeOperation = 'source-in';
        
        // Draw the foreground (the user)
        this.ctx.drawImage(results.image, 0, 0, this.canvas.width, this.canvas.height);

        // Only draw background behind the user
        this.ctx.globalCompositeOperation = 'destination-over';

        if (this.mode === 'blur') {
            this.ctx.filter = 'blur(10px)';
            this.ctx.drawImage(results.image, 0, 0, this.canvas.width, this.canvas.height);
            this.ctx.filter = 'none';
        } else if (this.mode === 'image' && this.backgroundImage) {
            this.ctx.drawImage(this.backgroundImage, 0, 0, this.canvas.width, this.canvas.height);
        } else {
             // Fallback or just plain video if something weird happens, though 'none' is handled above
             this.ctx.drawImage(results.image, 0, 0, this.canvas.width, this.canvas.height);
        }

        this.ctx.restore();
    }

    getStream() {
        return this.canvas.captureStream(30);
    }
}

export function createVideoProcessor() {
    return new VideoProcessor();
}
