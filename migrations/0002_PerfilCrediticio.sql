CREATE TABLE PerfilCrediticio(
ID SERIAL PRIMARY KEY NOT NULL,
fkUsuario INT NOT NULL REFERENCES Usuario(ID),
DNI TEXT UNIQUE NOT NULL,
HistorialCrediticio TEXT NOT NULL,
ExtractoBancario TEXT NOT NULL,
ComprobanteDeIngreso TEXT NOT NULL,
DecripcionFinanciera TEXT NOT NULL
)