CREATE TABLE Prestamo(
    ID SERIAL PRIMARY KEY NOT NULL,
    monto DECIMAL NOT NULL,
    fechaCreacion DATE NOT NULL,
    interes FLOAT NOT NULL,
    plazoPago DATE NOT NULL, 
    intervaloPago TEXT NOT NULL,
    riesgo int NOT NULL,
    fkPrestatario INT NOT NULL REFERENCES Usuario(ID),
    fkPrestamista INT NOT NULL REFERENCES Usuario(ID)
)