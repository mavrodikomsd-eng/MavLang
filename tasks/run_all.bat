@echo off
chcp 65001 >nul
set ROOT=C:\1\аап
set MAVC=C:\Users\admin\.cargo\bin\mavc.exe
set EXE=%ROOT%\target\debug\mavapp.exe
set SRC=%ROOT%\source\main.mavl
set TASKS=%ROOT%\tasks

echo === TESTING ALL 12 TASKS ===
echo.

set pass=0
set total=0

:: Helper: run one test
:: %1=name %2=mavlfile %3=mem %4=stdin (or -) %5=args (or -) %6=expected

:test1
copy /Y "%TASKS%\01_fizzbuzz.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail1
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
set expected=1^
2^
fizz^
4^
buzz^
fizz^
7^
8^
fizz^
buzz^
11^
fizz^
13^
14^
fizzbuzz
if "%out%"=="%expected%" (echo PASS 01_fizzbuzz & set /a pass+=1) else (echo FAIL 01_fizzbuzz: got^="%out%" expected^="%expected%")
set /a total+=1
goto test2

:fail1
echo FAIL 01_fizzbuzz: build failed
set /a total+=1

:test2
copy /Y "%TASKS%\02_csvsum.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail2
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="60" (echo PASS 02_csvsum & set /a pass+=1) else (echo FAIL 02_csvsum: got^="%out%" expected^="60")
set /a total+=1
goto test3

:fail2
echo FAIL 02_csvsum: build failed
set /a total+=1

:test3
copy /Y "%TASKS%\03_wordcount.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail3
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
set expected=3^
HELLO
if "%out%"=="%expected%" (echo PASS 03_wordcount & set /a pass+=1) else (echo FAIL 03_wordcount: got^="%out%" expected^="3^&HELLO^&")
set /a total+=1
goto test4

:fail3
echo FAIL 03_wordcount: build failed
set /a total+=1

:test4
copy /Y "%TASKS%\04_fileroundtrip.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail4
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="50" (echo PASS 04_fileroundtrip & set /a pass+=1) else (echo FAIL 04_fileroundtrip: got^="%out%" expected^="50")
set /a total+=1
goto test5

:fail4
echo FAIL 04_fileroundtrip: build failed
set /a total+=1

:test5
copy /Y "%TASKS%\05_args.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail5
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="none" (echo PASS 05_args_none & set /a pass+=1) else (echo FAIL 05_args_none: got^="%out%" expected^="none")
set /a total+=1

"%EXE%" yo > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="yo" (echo PASS 05_args_yo & set /a pass+=1) else (echo FAIL 05_args_yo: got^="%out%" expected^="yo")
set /a total+=1
goto test6

:fail5
echo FAIL 05_args: build failed
set /a total+=2
goto test6

:test6
copy /Y "%TASKS%\06_counter.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail6
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="5" (echo PASS 06_counter & set /a pass+=1) else (echo FAIL 06_counter: got^="%out%" expected^="5")
set /a total+=1
goto test7

:fail6
echo FAIL 06_counter: build failed
set /a total+=1

:test7
copy /Y "%TASKS%\07_fib.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail7
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="55" (echo PASS 07_fib & set /a pass+=1) else (echo FAIL 07_fib: got^="%out%" expected^="55")
set /a total+=1
goto test8

:fail7
echo FAIL 07_fib: build failed
set /a total+=1

:test8
copy /Y "%TASKS%\08_normalize.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail8
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="rust lang" (echo PASS 08_normalize & set /a pass+=1) else (echo FAIL 08_normalize: got^="%out%" expected^="rust lang")
set /a total+=1
goto test9

:fail8
echo FAIL 08_normalize: build failed
set /a total+=1

:test9
copy /Y "%TASKS%\09_borrowseries.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail9
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
set expected=5^
6
if "%out%"=="%expected%" (echo PASS 09_borrowseries & set /a pass+=1) else (echo FAIL 09_borrowseries: got^="%out%" expected^="5^&6^&")
set /a total+=1
goto test10

:fail9
echo FAIL 09_borrowseries: build failed
set /a total+=1

:test10
copy /Y "%TASKS%\10_ptroffset.mavl" "%SRC%" >nul
"%MAVC%" build --mem=manual >nul 2>&1
if errorlevel 1 goto fail10
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="99" (echo PASS 10_ptroffset & set /a pass+=1) else (echo FAIL 10_ptroffset: got^="%out%" expected^="99")
set /a total+=1
goto test11

:fail10
echo FAIL 10_ptroffset: build failed
set /a total+=1

:test11
copy /Y "%TASKS%\11_inputmath.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail11
echo 21 | "%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="42" (echo PASS 11_inputmath & set /a pass+=1) else (echo FAIL 11_inputmath: got^="%out%" expected^="42")
set /a total+=1
goto test12

:fail11
echo FAIL 11_inputmath: build failed
set /a total+=1

:test12
copy /Y "%TASKS%\12_repconcat.mavl" "%SRC%" >nul
"%MAVC%" build --mem=borrow >nul 2>&1
if errorlevel 1 goto fail12
"%EXE%" > "%ROOT%\tmp_out.txt" 2>&1
set /p out=<"%ROOT%\tmp_out.txt"
if "%out%"=="ababab" (echo PASS 12_repconcat & set /a pass+=1) else (echo FAIL 12_repconcat: got^="%out%" expected^="ababab")
set /a total+=1
goto done

:fail12
echo FAIL 12_repconcat: build failed
set /a total+=1

:done
echo.
echo Итого: %pass% / 12 прошли полностью

:: Restore original
copy /Y "%ROOT%\source\main.mavl.bak" "%SRC%" >nul
del "%ROOT%\tmp_out.txt" 2>nul
