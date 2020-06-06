echo "START EXPERIMENT" >> time.log; 
echo `date` >> time.log; 
for ((a=6; a <= 10; a++))
do
    echo $((1000000*a)) >> time.log; 
    cargo run --release $((1000000*a)) >> time.log;
done
echo "END EXPERIMENT" >> time.log
