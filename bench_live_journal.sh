echo "START EXPERIMENT" >> time.log; 
echo `date` >> time.log; 
for ((a=1; a <= 10; a++))
do
    echo $((100000*a)) >> time.log; 
    cargo run --release $((100000*a)) < ../data/rand-LiveJournal.txt >> time.log;
done
echo "END EXPERIMENT" >> time.log
