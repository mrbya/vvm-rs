module delayed_sequence (
    output logic [7:0] value,
    output logic       done
);

timeunit 1ns;
timeprecision 1ns;

initial begin
    value = 8'h00;
    done  = 1'b0;

    #2 value = 8'h11;
    #3 value = 8'h22;

    #5 begin
        value = 8'h33;
        done  = 1'b1;
    end
end

endmodule
