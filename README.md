# temples
Visualize LDS temple schedules.

It's harder than ever to get an appointment at temples of the Church of Jesus Christ of Latter-Day Saints. I thought it would be interesting to visualize how
many endowment seats are available for upcoming sessions.

## Pre-requisites
You must have an LDS account and Chrome installed on your local machine.

## Running
`USERNAME=<your LDS username> PASSWORD=<your LDS password> cargo run --release`

## Viewing Output
An HTML file will be generated with a grid of upcoming seats for sessions. I've found it's easiest to open this in Chrome, then convert it to PDF using the `GoFullPage - Full Page Screen Capture`
extension. Then it can be converted to other formats from there. Here are some sample outputs from the tool.

![Oakland Temple schedule](./sample_output/Oakland.png?raw=true)

![Logan Temple schedule](./sample_output/Logan.png?raw=true)
