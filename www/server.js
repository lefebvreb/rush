const express = require('express')
const cors = require('cors')
const path = require('path')

const app = express()
const port = process.env.PORT || 5050

app.use(cors())
app.use(express.static('public'))

app.get('*', (req, res) => {
    res.sendFile(path.resolve(__dirname, 'public', 'index.html'))
})

app.listen(port, () => {
    console.log(`Server is up @ http://localhost:${port}`)
})