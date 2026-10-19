# Papéis semânticos e modos

O exemplo com `primary` e `secondary`, cada um associado às cores padrão, `pastel` e `escuro`, está correto. Cada papel representa uma função; cada modo escolhe a cor que desempenha essa mesma função naquele contexto.

- Sem modo · padrão: cor usada quando não há um modo solicitado ou quando esse modo não tem substituição para o papel.
- Modo com associação: usa a cor escolhida naquele modo.
- Modo com associação vazia explícita: o papel fica sem associação naquele modo até uma cor ser escolhida. Não usa automaticamente a cor padrão.

Usar duas cores distintas chamadas “Monocromática 1” é válido: as associações usam seus IDs, não o nome. Renomeá-las para algo como “primary escuro” e “secondary escuro” pode facilitar a identificação, sem mudar seu funcionamento.

Essas associações não recalculam os valores das cores. São informações publicadas com a paleta para que um consumidor compatível resolva o papel e o modo desejados. Um programa que use somente as amostras simples de cor pode ignorar essa camada. Não é necessário criar uma relação documental adicional para que uma associação de papel funcione.
