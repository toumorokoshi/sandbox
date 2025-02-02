package main

import "fmt"
import "sync"


func main() {
	var wg sync.WaitGroup
	channel := make(chan int)
	counter := 0
	// range would work here, but the compiler
	// complains about an unused variable.
	for i := 0; i < 1000; i+= 1 {
		wg.Add(1)
		go func() {
			for j := 0; j < 1000; j+=1 {
				channel <- 1
			}
			wg.Done()
		}()
	}
	// run our collector goroutine
	go func() {
		wg.Add(1)
		for j := 0; j < 1000*1000; j += 1 {
			counter += <- channel
		}
		wg.Done()
	}()
	wg.Wait()
	fmt.Println(counter)
}
