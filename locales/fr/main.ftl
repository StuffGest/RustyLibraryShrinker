# CLI Clap
cli-about = Compresseur de BD, Mangas et Comics vers CBZ
cli-help-input = Chemin vers le fichier unique ou le répertoire contenant les archives à traiter
cli-help-lang = Langue de l'interface (ex: "fr", "en"). Par défaut "fr"
cli-help-quality = Niveau de qualité pour l'encodage WebP (de 1 à 100). Une valeur de 90 offre un excellent rapport poids/qualité
cli-help-height = Hauteur cible en pixels pour le redimensionnement des pages
cli-help-dim = Dimension maximale de sécurité utilisée en cas d'impossibilité de déterminer le ratio
cli-help-mode = Mode de gestion des fichiers de sortie (suffix, rename, replace)
cli-help-threads = Nombre de threads maximum (0 pour auto)
cli-help-glob = Utilisation d'un motif de recherche Glob pour filtrer les fichiers (ex: "**/Batman*.cbr")
cli-help-savings = Pourcentage minimal de gain de poids requis pour valider le remplacement du fichier original
cli-help-log = Chemin vers le fichier de log
cli-help-verbose = Affiche plus d'informations dans la console durant l'exécution
cli-help-skip = Désactive la compression des images : convertit uniquement le conteneur vers le format CBZ
cli-help-force = Force le ré-encodage et le redimensionnement des images même si elles sont déjà au format WebP

# Libellés UI pour la barre de progression
cli-label-files = fichiers
cli-label-elapsed = Temps écoulé
cli-label-remaining = Restant

# Messages UI & Logs
msg-log-start = 🚀 Démarrage de RustyLibraryShrinker
msg-no-files-found = Aucun fichier trouvé.
msg-start-processing = RustyLibraryShrinker : { $count } fichier(s) à traiter
msg-image-skipped = IMAGE SKIPPED
msg-reason = Raison
msg-reason-decode = Échec du décodage de l'image (fichier peut-être corrompu)
msg-log-no-gain = gain insuffisant
msg-processing-complete = Traitement terminé !

# Rapport final & Statistiques
msg-detailed-results = DÉTAIL PAR FICHIER
msg-skipped-no-gain = pas de gain
msg-global-summary = RÉSUMÉ GLOBAL
msg-summary-total-files = Fichiers total
msg-optimized = Optimisés
msg-not-optimized = Non optimisés
msg-failed = Échecs
msg-summary-img-optimized = Images optimisées
msg-summary-img-skipped = Images ignorées
msg-original-size = Taille originale
msg-final-size = Taille finale
msg-total-gain = Gain total