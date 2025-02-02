package main

import "fmt"
import "sync/atomic"
import "sync"

func main() {
	var counter atomic.Int32
	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			for j := 0; j < 100; j++ {
				counter.Add(1)
			}
			wg.Done()
		}()
	}
	wg.Wait()
	fmt.Println(counter.Load())
}
