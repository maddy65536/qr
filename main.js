import init, { QrCodeGenerator, QrCodeArgs, ImageArgs } from "./qr-wasm/pkg/qr_wasm.js"


const textentry = document.getElementById("textentry");
const thebutton = document.getElementById("thebutton");
const thebutton2 = document.getElementById("thebutton2");
const output = document.getElementById("output");
const canvas = document.getElementById("outputCanvas");
const fileinput = document.getElementById("fileinput");


// Declare the generator variable in the outer scope
let generator;

export function generate() {
    if (!generator) {
        console.error("WASM module is not initialized yet!");
        return;
    }

    let qr_args = new QrCodeArgs(textentry.value, null, null, null);
    let qr_data = generator.generate_qr_code(qr_args);

    let ctx = canvas.getContext("2d");
    let [width, height] = [qr_data.width(), qr_data.height()];
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

    let qr_args = new QrCodeArgs(textentry.value, null, null, 15);
    let img_args = new ImageArgs(10, null, null, null);
    let qr_data = generator.generate_qr_code_image(qr_args, img_args);

    let ctx = canvas.getContext("2d");
    let [width, height] = [qr_data.width(), qr_data.height()];
    canvas.width = width;
    canvas.height = height;
    const clampedArray = new Uint8ClampedArray(qr_data.data());
    const imageData = new ImageData(clampedArray, width, height);
    ctx.putImageData(imageData, 0, 0);
}

async function run() {
    await init();
    generator = new QrCodeGenerator();
    thebutton.addEventListener("click", generate);
    thebutton2.addEventListener("click", generate_image);
    fileinput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (!file) return;

        console.log("somethng happened")

        const reader = new FileReader();
        reader.onload = (e) => {
            const arrayBuffer = e.target.result;
            const uint8Array = new Uint8Array(arrayBuffer);
            generator.set_image(uint8Array);
        };

        reader.readAsArrayBuffer(file);
    });
}

run();
