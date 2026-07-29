`timescale 1ns/1ps

// Behavioral single-frame transmitter for timing-scheduler verification.
module timed_uart (
    input logic [7:0] data,
    input logic parity_enable,
    input logic odd_parity,
    input logic inject_parity_error,
    input logic inject_stop_error,
    output logic tx,
    output logic busy,
    output logic done
);
    // Latched parity value emitted after the eight data-bit intervals.
    logic parity;

    initial begin
        tx = 1'b1;
        busy = 1'b1;
        done = 1'b0;
        // XOR reduction is even parity; odd mode inverts that transmitted bit.
        parity = ^data ^ odd_parity;

        // Start bit, then eight least-significant-bit-first data intervals.
        #10 tx = 1'b0;
        #10 tx = data[0];
        #10 tx = data[1];
        #10 tx = data[2];
        #10 tx = data[3];
        #10 tx = data[4];
        #10 tx = data[5];
        #10 tx = data[6];
        #10 tx = data[7];

        if (parity_enable) begin
            // Error injection corrupts the line, not the internally calculated parity.
            #10 tx = parity ^ inject_parity_error;
        end

        // A low injected stop bit creates a framing error for the checker.
        #10 tx = ~inject_stop_error;
        #10 tx = 1'b1;
        busy = 1'b0;
        done = 1'b1;
    end
endmodule
