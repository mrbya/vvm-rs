// Fixed-depth one-clock FIFO with registered data and one-cycle error flags.
module sync_fifo (
    input  logic       clk,
    input  logic       reset_n,
    input  logic       push,
    input  logic [7:0] push_data,
    input  logic       pop,
    output logic [7:0] pop_data,
    output logic       empty,
    output logic       full,
    output logic [3:0] occupancy,
    output logic       overflow,
    output logic       underflow
);
    // Storage and circular indices are implementation details; the Rust model
    // deliberately checks queue behavior instead.
    logic [7:0] memory [0:7];
    logic [2:0] write_pointer;
    logic [2:0] read_pointer;

    // Flags are combinational views of registered occupancy.
    assign empty = occupancy == 0;
    assign full = occupancy == 8;

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            write_pointer <= 0;
            read_pointer <= 0;
            occupancy <= 0;
            pop_data <= 0;
            overflow <= 0;
            underflow <= 0;
        end else begin
            // Error indicators describe a request rejected at this edge.
            overflow <= push && full;
            underflow <= pop && empty;

            case ({push && !full, pop && !empty})
                2'b10: begin
                    memory[write_pointer] <= push_data;
                    write_pointer <= write_pointer + 1'b1;
                    occupancy <= occupancy + 1'b1;
                end
                2'b01: begin
                    pop_data <= memory[read_pointer];
                    read_pointer <= read_pointer + 1'b1;
                    occupancy <= occupancy - 1'b1;
                end
                2'b11: begin
                    // Both pointers advance; occupancy therefore holds steady.
                    memory[write_pointer] <= push_data;
                    pop_data <= memory[read_pointer];
                    write_pointer <= write_pointer + 1'b1;
                    read_pointer <= read_pointer + 1'b1;
                end
                default: begin
                end
            endcase
        end
    end
endmodule
