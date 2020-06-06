echo "START EXPERIMENT" >> time.log; 
echo `date` >> time.log; 
for ((a=1; a <= 10; a++))
do
    echo $((1000000*a)) >> time.log; 
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
    cargo run --release $((1000000*a)) < ../data/LiveJournal-rand.txt >> time.log;
done
echo "END EXPERIMENT" >> time.log
