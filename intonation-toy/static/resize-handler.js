// Handle canvas resizing to maintain responsive square aspect ratio
function resizeCanvas() {
    const canvas = document.getElementById('wgpu-canvas');
    if (!canvas) return;
    
    const menuBarHeight = 60;
    const padding = 40; // 20px on each side
    const minSize = 256;
    const maxSize = 1024;
    
    // Calculate available space
    const availableWidth = window.innerWidth - padding;
    const availableHeight = window.innerHeight - menuBarHeight - padding;
    
    // Take the smaller dimension to maintain square aspect ratio
    let size = Math.min(availableWidth, availableHeight, maxSize);
    size = Math.max(size, minSize);
    
    // Set both CSS dimensions and canvas attributes
    canvas.style.width = size + 'px';
    canvas.style.height = size + 'px';
    canvas.width = size;
    canvas.height = size;
}

// Initial resize
window.addEventListener('load', resizeCanvas);

// Handle window resize
window.addEventListener('resize', resizeCanvas);

// Also call on DOMContentLoaded to ensure early sizing
document.addEventListener('DOMContentLoaded', resizeCanvas);