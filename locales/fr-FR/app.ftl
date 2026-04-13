app-title = Compagnon Turn Checker
app-server-starting = Démarrage du serveur de synchronisation local...
app-launch-failed = Échec du lancement de l'interface : { $error }
app-theme-toggle-tooltip = Basculer entre le mode clair et sombre
app-always-on-top-disable-tooltip = Désactiver le mode toujours au premier plan
app-always-on-top-enable-tooltip = Garder l'application au-dessus des autres fenêtres
app-minimal-mode-disable-tooltip = Revenir à la vue complète
app-minimal-mode-enable-tooltip = Passer à une surcouche compacte

server-stopped-log = Le serveur de synchronisation s'est arrêté : { $error }
server-invalid-bind-address = Adresse de liaison invalide : { $error }

notification-url-not-configured = L'URL de notification push n'est pas configurée.
notification-device-token-unavailable = Le jeton de l'appareil n'est pas encore disponible. Associez l'application iOS et acceptez les autorisations de notifications push.
notification-title-new-turn = Nouveau tour
notification-body-new-turn = L'action de nouveau tour a été reçue.
notification-bearer-token-missing = Le jeton d'authentification des notifications push n'est pas configuré. Définissez { $env } ou pointez { $file_env } vers un fichier contenant le jeton, ou intégrez-le à la compilation via { $compile_env }.
notification-bearer-token-file-read-failed = Impossible de lire le fichier du jeton d'authentification des notifications push à { $path } : { $error }

startup-waiting = En attente...
startup-failed = Échec du démarrage.
startup-unsent-data-title = Données non envoyées détectées
startup-unsent-data-message =
    { $count ->
        [one] La base de données actuelle contient { $count } élément non envoyé.
       *[other] La base de données actuelle contient { $count } éléments non envoyés.
    }
startup-keep-db-question = Voulez-vous conserver la base de données actuelle ?
startup-keep-db-button = Conserver la base actuelle
startup-reset-db-button = Supprimer et recréer la base

pairing-title = Scanner pour connecter
pairing-description = Ouvrez l'application iOS et scannez le QR code pour configurer l'adresse du serveur.
pairing-qr-failed = Échec de la génération du QR code d'association.
pairing-server-url = URL du serveur
pairing-server-connection-missing = Les informations de connexion du serveur ne sont pas disponibles.

action-cancel = Annuler
action-save = Enregistrer
action-back = Retour
action-restart = Redémarrer
action-next-turn = Tour suivant

dialog-new-turn-title = Nouveau tour
dialog-new-turn-pending-message =
    { $count ->
        [one] Il vous reste { $count } Check obligatoire du tour en cours à terminer.
       *[other] Il vous reste { $count } Checks obligatoires du tour en cours à terminer.
    }
dialog-new-turn-blocked-message = La notification de nouveau tour ne peut pas être envoyée tant que toutes les checks obligatoires du tour en cours ne sont pas cochées.
dialog-new-turn-confirm-message = L'envoi d'une notification de nouveau tour fera passer cet écran en mode attente jusqu'à l'arrivée du prochain tour.
dialog-restart-title = Redémarrer
dialog-restart-unsent-message =
    { $count ->
        [one] La base de données contient { $count } élément non envoyé.
       *[other] La base de données contient { $count } éléments non envoyés.
    }
dialog-restart-confirm-message = Le redémarrage supprimera puis recréera la base de données, puis reviendra à l'écran d'association.

content-error-no-current-session = Aucune session en cours n'est encore disponible.
content-new-check-button = Nouveau Check
content-source-game-turns-button = Checks des tours du jeu
content-source-game-button = Checks du jeu
content-source-template-button = Checks du modèle
content-comments-button = Commentaires
content-missing-comment-slot = Emplacement de commentaire manquant pour { $comment_type }.

waiting-next-turn-title = En attente du prochain tour...
waiting-next-turn-description = L'application se déverrouillera automatiquement lorsque le nouveau tour sera reçu.

comments-title = Commentaires
comment-type-game = Jeu
comment-type-turn = Tour
comments-no-slot = Aucun emplacement de commentaire n'est disponible.

checklist-turn-label = Tour { $turn }
checklist-current-turn = Tour en cours
checklist-empty = Aucun Check pour l'instant.

source-checks-empty = Aucun Check trouvée pour { $title }.

new-check-title = Créer une nouveau Check
field-name = Nom
field-detail = Détail
field-source = Source
field-tag = Étiquette
field-repeat = Répétition
field-repeat-value = Valeur de répétition
field-mandatory = Obligatoire
field-no-tag = Aucune étiquette

source-game = Jeu
source-global-game = Jeu global
source-blueprint = Modèle
source-turn = Tour

repeat-everytime = À chaque fois
repeat-conditional = Conditionnel
repeat-specific = Spécifique
repeat-until = Jusqu'à

check-mandatory = Obligatoire
repeat-badge-every-turn = Chaque tour
repeat-badge-conditional = Conditionnel (Tour { $turn })
repeat-badge-specific = Spécifique (Tour { $turn })
repeat-badge-until = Jusqu'au tour { $turn }

validation-name-required = Le nom est obligatoire.
validation-field-valid-integer = { $field } doit être un entier valide.
validation-field-at-least = { $field } doit être au moins égale à { $min }.
