import * as wasm from "rustendo";

const loadCartridgeButton = document.getElementById('load-cartridge-button');
const cartridgeFile = document.getElementById('cartridge-file');

loadCartridgeButton.addEventListener('click', function() {
    cartridgeFile.click();
}, false);


cartridgeFile.addEventListener('change', function() {
    if (this.files.length === 0) {
        return;
    }

    const cartridge = this.files[0];

    cartridge.arrayBuffer().then(function(arrayBuffer) {
        const byteArray = new Uint8Array(arrayBuffer);
        wasm.render(byteArray);
    });
}, false);
