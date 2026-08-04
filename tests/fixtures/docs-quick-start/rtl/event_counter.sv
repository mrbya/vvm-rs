// ANCHOR: rtl
module event_counter (
    input logic clk,
    input logic reset_n,
    input logic event_pulse,
    output logic [7:0] total
);

always_ff @(posedge clk or negedge reset_n) begin
    if (!reset_n) begin
        total <= '0;
    end else if (event_pulse) begin
        total <= total + 1'b1;
    end
end

endmodule
// ANCHOR_END: rtl
