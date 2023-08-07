CREATE TABLE Prestamo(
    ID BIGSERIAL PRIMARY KEY NOT NULL,
    monto DECIMAL NOT NULL CHECK (monto > 0),
    fechaCreacion DATE NOT NULL,
    interes FLOAT NOT NULL,
    plazoPago DATE NOT NULL, 
    intervaloPago TEXT NOT NULL,
    riesgo INT NOT NULL CHECK (riesgo > 0),
    fkPrestatario INT REFERENCES Usuario(ID),
    fkPrestamista INT REFERENCES Usuario(ID),
    CHECK (fkPrestatario IS NOT NULL OR fkPrestamista IS NOT NULL)
)