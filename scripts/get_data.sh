#/bin/sh

rm -rf data/*

curl https://raw.githubusercontent.com/dwyl/english-words/master/words.txt -o data/bank.txt

python3 scripts/generate-bank.py data/bank.txt 4  >>  data/ve-4.txt
python3 scripts/generate-bank.py data/bank.txt 5  >>  data/ve-5.txt
python3 scripts/generate-bank.py data/bank.txt 6  >>  data/e-6.txt
python3 scripts/generate-bank.py data/bank.txt 7  >>  data/e-7.txt
python3 scripts/generate-bank.py data/bank.txt 8  >>  data/e-8.txt
python3 scripts/generate-bank.py data/bank.txt 9  >>  data/a-9.txt
python3 scripts/generate-bank.py data/bank.txt 10 >>  data/a-10.txt
python3 scripts/generate-bank.py data/bank.txt 11 >>  data/h-11.txt
python3 scripts/generate-bank.py data/bank.txt 12 >>  data/h-12.txt
python3 scripts/generate-bank.py data/bank.txt 13 >>  data/vh-13.txt
python3 scripts/generate-bank.py data/bank.txt 14 >>  data/vh-14.txt
python3 scripts/generate-bank.py data/bank.txt 15 >>  data/vh-15.txt

