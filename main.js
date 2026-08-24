import init, { QrCodeGenerator, QrCodeArgs, ImageArgs } from "./qr-wasm/pkg/qr_wasm.js"

// output stuff
const canvas = document.getElementById("outputCanvas");
const errorOutput = document.getElementById("errorOutput");
const updateButton = document.getElementById("updateButton");
const saveButton = document.getElementById("saveButton");
const copyButton = document.getElementById("copyButton");

// control stuff
const textentry = document.getElementById("textentry");
const ecSelect = document.getElementById("ecSelect");
const maskSelect = document.getElementById("maskSelect");
const versionNumber = document.getElementById("versionNumber");

// image stuff
const imageEnable = document.getElementById("imageEnable");
const fileinput = document.getElementById("fileinput");
const xOffsetNumber = document.getElementById("xOffsetNumber");
const yOffsetNumber = document.getElementById("yOffsetNumber");
const threshholdNumber = document.getElementById("threshholdNumber");
const scaleNumber = document.getElementById("scaleNumber");


function clamp(num, low, high) {
    return Math.max(low, Math.min(high, num))
}

// Declare the generator variable in the outer scope
let generator;

export function generate() {
    errorOutput.innerText = "";

    if (!generator) {
        console.error("WASM module is not initialized yet!");
        return;
    }

    let mask = null;
    if (mask !== "") {
        mask = Number(maskSelect.value);
    }
    const version = clamp(Number(versionNumber.value), 1, 40);
    const qr_args = new QrCodeArgs(textentry.value, ecSelect.value, mask, version);

    let image_args = null;
    if (imageEnable.checked) {
        if (!generator.has_image()) {
            errorOutput.innerText = "Image enabled, but no image uploaded!";
            return;
        }

        let x_offset = Number(xOffsetNumber.value);
        let y_offset = Number(yOffsetNumber.value);
        let threshold = clamp(Number(threshholdNumber.value), 0, 255);
        let scale = parseFloat(scaleNumber.value);

        image_args = new ImageArgs(x_offset, y_offset, threshold, scale);
        console.log("meow");
    }

    let qr_data;
    try {
        qr_data = generator.generate_qr_code(qr_args, image_args);
    } catch (err) {
        errorOutput.innerText = err.message;
        return;
    }

    const ctx = canvas.getContext("2d");
    const [width, height] = [qr_data.width(), qr_data.height()];
    canvas.width = width;
    canvas.height = height;
    const clampedArray = new Uint8ClampedArray(qr_data.data());
    const imageData = new ImageData(clampedArray, width, height);
    ctx.putImageData(imageData, 0, 0);
}

export function generate_image() {
    if (!generator) {
        console.error("WASM module is not initialized yet!");
        return;
    }

    const qr_args = new QrCodeArgs(textentry.value, null, null, 15);
    const img_args = new ImageArgs(10, null, null, null);
    const qr_data = generator.generate_qr_code(qr_args, img_args);

    const ctx = canvas.getContext("2d");
    const [width, height] = [qr_data.width(), qr_data.height()];
    canvas.width = width;
    canvas.height = height;
    const clampedArray = new Uint8ClampedArray(qr_data.data());
    const imageData = new ImageData(clampedArray, width, height);
    ctx.putImageData(imageData, 0, 0);
}

function saveImage() {
    // lol
    canvas.toBlob(blob => {
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.download = 'qr.png';
        a.href = url;
        a.click();
    })
}

function copyImage() {
    canvas.toBlob(blob => {
        const item = new ClipboardItem({ "image/png": blob });
        navigator.clipboard.write([item]);
    })
}

async function run() {
    await init();
    generator = new QrCodeGenerator();

    updateButton.addEventListener("click", generate);
    saveButton.addEventListener("click", saveImage);
    copyButton.addEventListener("click", copyImage);

    fileinput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (!file) return;

        const reader = new FileReader();
        reader.onload = (e) => {
            const arrayBuffer = e.target.result;
            const uint8Array = new Uint8Array(arrayBuffer);
            generator.set_image(uint8Array);
        };

        reader.readAsArrayBuffer(file);
    });

    generate();
}

run();
