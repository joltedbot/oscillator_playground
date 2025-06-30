# Oscillator Playground

This is not so much a real project as it is place for me to learn about DSP in general and in Rust specifically.

It is not a production audio appliction but I beleive it might prove useful to others.

It will change over time as I add more stuff.

If you are looking to make a serious audio application I suggest you look at some of the great crates that already exist by people that know what 
they are doing already.  

To use it you can just "cargo run" or "cargo build" and run the binary as usual.  There are no flags or cli parameters.

If set it up you can play around with the code in main.rs  The primary targets being:

- There are 2 constants at the top of main.rs to set the standard output level and how long it loops the sequence before exiting.  The output level isn't entirely accurate. The oscillators output -dbfs and the this level adjusts with that in mind but way the oscillators combine will deviate from that assumption so play with it as needed.
  
  `const OUTPUT_LEVEL: f32 = -10.0;
  const MAIN_LOOP_DURATION_SECONDS: u64 = 60;`

- Setup the oscilators to use with these lines. The shape options are in oscillators.rs
  
  `oscillators.set_oscillator1_type(WaveShape::Square);  
  oscillators.set_oscillator2_type(WaveShape::Square);  
  oscillators.set_oscillator3_type(WaveShape::Square);`
  
  and uncomment and set this
  
  `oscillators.enable_unison(0.03);`
  
  You can also tweak the oscilators where they are being called. The 1.0 parameter is for setting the relative level between the oscilators. 1.0 being full volume, 0.0 being no volume and you can play with the values in between.  The last parameter is a modulation option. For square wave it is PWM, for Noise it is tremolo, and for the rest it is frequency modulation. None means no modulation obviously and You can pass something in what ever you like with "Some(??)" as with oscillator 2 in this example passing in the lfo.  But you can pass in static numbers or other oscilators or whatever.
  
  `let oscillator1_sample = oscillator.get_oscillator1_next_sample(note_frequency, 1.0, None);  
  let oscillator2_sample = oscillator.get_oscillator2_next_sample(note_frequency, 1.0, Some(lfo1_value));  
  let oscillator3_sample = oscillator.get_oscillator3_next_sample(note_frequency, 1.0, None);`

- You can setup a basic note sequence with this line. The numbers in the vec are midi note numbers. You can use more or less notes and note 0 is a rest.
  
  `let mut sequencer = Sequencer::new(vec![60, 62, 63, 65, 67, 68, 70, 72]);`

- Modify the ADSR envelope per note using these
  
  `modulation.set_adsr_attack_milliseconds(50);
    modulation.set_adsr_decay_milliseconds(300);
    modulation.set_adsr_release_milliseconds(200);
    modulation.set_adsr_sustain_length_milliseconds(300);
    modulation.set_adsr_sustain_level(OUTPUT_LEVEL - 6.0);`

- You can tweak and use the LFO using this line. The 3 parameters are the frequency in Hz, the center value for the lfo, and the total swing around that center value. (e.g. for these values it will range from 0.55 to 0.8.5 at 3Hz)
  
  `let lfo1_value = lfo1.get_next_value(3.0, 0.7, 0.3);`
  
  If you want to add another LFO you can duplicate that line and this one near the top of the main function. Just change lfo1 to lfo2 and so on.
  
  `let mut lfo1 = LFO::new(sample_rate);`
  
  
  
  