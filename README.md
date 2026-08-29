# oolio151-nes
An emulator of the Nintendo Entertainment System, built in Rust. Currently in Development.

### Development Steps
<ul>
    <li><s>CPU / official opcodes - in progress</li>
    <ul>
      <li>interrupts</li>
      <li> unofficial opcodes </s> (some bugs remain) <s> </li>
    </ul>
    <li>Bus and Mapper, Cartridge Loading</li>
    <li>PPU</li>
    <li>Display</li>
    <li>Input</li></s>
    <li><b>APU << DOING</b></li>
    <li>MMC3, MMC5, other mappers</li>
    
</ul>

#### only roms that use mapper 0 / NROM are supported rn, more to come
#### cpu tests are from [SingleStepTests/65x02](https://github.com/SingleStepTests/65x02)
#### passed 75/141 tests from [100thCoin/AccuracyCoin](https://github.com/100thCoin/AccuracyCoin)